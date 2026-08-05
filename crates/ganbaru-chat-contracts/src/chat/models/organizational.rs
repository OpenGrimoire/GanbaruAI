use super::{
    ChatAgentRunId, ChatAttachmentId, ChatChannelId, ChatConversationId, ChatConversationItemId,
    ChatMessageRevisionId, ChatParticipantId, ChatReplyThreadId, ChatScheduledMessageId,
    ChatTeammatePolicyRevisionId, ChatThreadId, ChatTurnId, ChatWorkAssignmentId, ModelId,
    ModelOptionSelection, ProjectWorkingFolderId, ProviderInstanceId, UtcTimestamp, VersionedJson,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatParticipantKind {
    LocalUser,
    AiTeammate,
    Human,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatTeammateConfigurationState {
    Healthy,
    NeedsSetup,
    ProviderUnavailable,
    FolderAccessMissing,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatWorkAssignmentState {
    Queued,
    Working,
    WaitingForAnswer,
    WaitingForApproval,
    ReadyForReview,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatApprovalPolicy {
    AskForApproval,
    ApproveForMe,
    FullAccess,
    Custom,
}

impl ChatWorkAssignmentState {
    /// Returns whether the assignment still occupies the thread's active work slot.
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Queued
                | Self::Working
                | Self::WaitingForAnswer
                | Self::WaitingForApproval
                | Self::ReadyForReview
        )
    }

    /// Returns whether a new message can still be delivered to this assignment.
    pub fn accepts_continuation(self) -> bool {
        matches!(
            self,
            Self::Queued | Self::Working | Self::WaitingForAnswer | Self::WaitingForApproval
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ChatWorkAssignmentState;

    #[test]
    fn review_work_is_active_but_requires_a_new_follow_up_assignment() {
        let state = ChatWorkAssignmentState::ReadyForReview;

        assert!(state.is_active());
        assert!(!state.accepts_continuation());
        assert!(ChatWorkAssignmentState::Working.accepts_continuation());
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatParticipantRead {
    pub id: ChatParticipantId,
    pub kind: ChatParticipantKind,
    pub display_name: String,
    pub handle: Option<String>,
    pub avatar: VersionedJson,
    pub revision: u64,
    pub archived_at: Option<UtcTimestamp>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatTeammatePolicyRead {
    pub id: ChatTeammatePolicyRevisionId,
    pub teammate_id: ChatParticipantId,
    pub revision: u64,
    pub provider_instance_id: ProviderInstanceId,
    pub provider_managed_model: bool,
    pub model_id: Option<ModelId>,
    pub model_options: Vec<ModelOptionSelection>,
    pub effort: Option<String>,
    pub speed: Option<String>,
    pub provider_options: VersionedJson,
    pub created_at: UtcTimestamp,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAiTeammateRead {
    pub participant: ChatParticipantRead,
    pub purpose: String,
    pub instructions: String,
    pub configuration_state: ChatTeammateConfigurationState,
    pub latest_policy: Option<ChatTeammatePolicyRead>,
    pub channel_count: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatWorkingFolderGrantRead {
    pub working_folder_id: ProjectWorkingFolderId,
    pub display_name: String,
    pub is_default: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatConversationMembershipRead {
    pub conversation_id: ChatConversationId,
    pub participant: ChatParticipantRead,
    pub addressable: bool,
    pub approval_policy: ChatApprovalPolicy,
    pub working_folder_grants: Vec<ChatWorkingFolderGrantRead>,
    pub revision: u64,
    pub removed_at: Option<UtcTimestamp>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatProjectPrimaryWorkingFolderRead {
    pub project_id: String,
    pub working_folder_id: ProjectWorkingFolderId,
    pub revision: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatParticipantMentionRead {
    pub participant_id: ChatParticipantId,
    pub participant_kind: ChatParticipantKind,
    pub handle_snapshot: Option<String>,
    pub label_snapshot: String,
    pub start_offset: u64,
    pub end_offset: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResourceReferenceRead {
    pub working_folder_id: ProjectWorkingFolderId,
    pub kind: String,
    pub relative_path: String,
    pub display_label: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatReplyThreadSummaryRead {
    pub id: ChatReplyThreadId,
    pub reply_count: u64,
    pub last_activity_at: UtcTimestamp,
    pub participants: Vec<ChatParticipantRead>,
    pub unread: bool,
    pub work_state: Option<ChatWorkAssignmentState>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageRead {
    pub item_id: ChatConversationItemId,
    pub conversation_id: ChatConversationId,
    pub reply_thread_id: Option<ChatReplyThreadId>,
    pub revision_id: ChatMessageRevisionId,
    pub revision: u64,
    pub author: ChatParticipantRead,
    pub normalized_markdown: String,
    pub rich_content: VersionedJson,
    pub mentions: Vec<ChatParticipantMentionRead>,
    pub attachment_ids: Vec<ChatAttachmentId>,
    pub resource_references: Vec<ChatResourceReferenceRead>,
    pub reply_thread: Option<ChatReplyThreadSummaryRead>,
    pub ordinal: u64,
    pub edited_at: Option<UtcTimestamp>,
    pub created_at: UtcTimestamp,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatScheduledMessageRead {
    pub id: ChatScheduledMessageId,
    pub channel_id: ChatChannelId,
    pub reply_thread_id: Option<ChatReplyThreadId>,
    pub normalized_markdown: String,
    pub rich_content: VersionedJson,
    pub attachment_ids: Vec<ChatAttachmentId>,
    pub participant_mentions: Vec<ChatParticipantMentionRead>,
    pub resource_references: Vec<ChatResourceReferenceRead>,
    pub also_send_to_channel: bool,
    pub state: String,
    pub scheduled_for: UtcTimestamp,
    pub last_error: Option<String>,
    pub created_at: UtcTimestamp,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatScheduledMessageDispatchRead {
    pub processed_count: u32,
    pub dispatched_count: u32,
    pub dispatched_channel_ids: Vec<ChatChannelId>,
    pub next_dispatch_at: Option<UtcTimestamp>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatChannelPageRead {
    pub channel_id: ChatChannelId,
    pub messages: Vec<ChatMessageRead>,
    pub previous_cursor: Option<String>,
    pub revision: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatWorkAssignmentRead {
    pub id: ChatWorkAssignmentId,
    pub reply_thread_id: ChatReplyThreadId,
    pub teammate: ChatParticipantRead,
    pub triggering_message_item_id: ChatConversationItemId,
    pub previous_assignment_id: Option<ChatWorkAssignmentId>,
    pub state: ChatWorkAssignmentState,
    pub state_reason: Option<String>,
    pub revision: u64,
    pub settled_at: Option<UtcTimestamp>,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAgentRunRead {
    pub id: ChatAgentRunId,
    pub assignment_id: ChatWorkAssignmentId,
    pub project_id: String,
    pub working_folder_id: ProjectWorkingFolderId,
    pub teammate_policy_revision_id: ChatTeammatePolicyRevisionId,
    pub effort: Option<String>,
    pub provider_execution_turn_id: ChatTurnId,
    pub provider_execution_thread_id: Option<ChatThreadId>,
    pub state: String,
    pub run_ordinal: u64,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatReplyThreadPageRead {
    pub thread: ChatReplyThreadSummaryRead,
    pub root_message: ChatMessageRead,
    pub replies: Vec<ChatMessageRead>,
    pub assignment: Option<ChatWorkAssignmentRead>,
    pub agent_runs: Vec<ChatAgentRunRead>,
    pub previous_cursor: Option<String>,
    pub revision: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageSearchResultRead {
    pub project_id: String,
    pub channel_id: ChatChannelId,
    pub channel_name: String,
    pub conversation_id: ChatConversationId,
    pub reply_thread_id: Option<ChatReplyThreadId>,
    pub message_item_id: ChatConversationItemId,
    pub ordinal: u64,
    pub author_participant_id: ChatParticipantId,
    pub author_kind: ChatParticipantKind,
    pub author_display_name: String,
    pub excerpt: String,
    pub created_at: UtcTimestamp,
}
