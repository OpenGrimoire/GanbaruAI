//! Stable wire contracts for organizational Chat commands.

use super::super::models::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatTeammatePolicyInput {
    pub provider_instance_id: ProviderInstanceId,
    #[serde(default)]
    pub provider_managed_model: bool,
    pub model_id: Option<ModelId>,
    #[serde(default)]
    pub model_options: Vec<ModelOptionSelection>,
    pub effort: Option<String>,
    pub speed: Option<String>,
    pub provider_options: VersionedJson,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatTeammateMembershipInput {
    pub channel_id: ChatChannelId,
    #[serde(default = "default_addressable")]
    pub addressable: bool,
    pub approval_policy: ChatApprovalPolicy,
    pub working_folder_ids: Vec<ProjectWorkingFolderId>,
    pub default_working_folder_id: ProjectWorkingFolderId,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatTeammateCommand {
    pub teammate_id: ChatParticipantId,
    pub display_name: String,
    pub handle: String,
    pub avatar: VersionedJson,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub instructions: String,
    pub policy: ChatTeammatePolicyInput,
    #[serde(default)]
    pub memberships: Vec<ChatTeammateMembershipInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChatTeammateProfileCommand {
    pub teammate_id: ChatParticipantId,
    pub display_name: String,
    pub handle: String,
    pub avatar: VersionedJson,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub instructions: String,
    pub expected_revision: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishChatTeammatePolicyCommand {
    pub teammate_id: ChatParticipantId,
    pub policy: ChatTeammatePolicyInput,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertChatTeammateMembershipCommand {
    pub teammate_id: ChatParticipantId,
    pub membership: ChatTeammateMembershipInput,
    pub expected_revision: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatParticipantMentionInput {
    pub participant_id: ChatParticipantId,
    pub participant_kind: ChatParticipantKind,
    pub handle_snapshot: Option<String>,
    pub label_snapshot: String,
    pub start_offset: u64,
    pub end_offset: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResourceReferenceInput {
    pub working_folder_id: ProjectWorkingFolderId,
    pub kind: String,
    pub relative_path: String,
    pub display_label: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostChatMessageCommand {
    pub client_command_id: ChatCommandId,
    pub channel_id: ChatChannelId,
    pub reply_thread_id: Option<ChatReplyThreadId>,
    pub normalized_markdown: String,
    pub rich_content: VersionedJson,
    #[serde(default)]
    pub attachment_ids: Vec<ChatAttachmentId>,
    #[serde(default)]
    pub participant_mentions: Vec<ChatParticipantMentionInput>,
    #[serde(default)]
    pub resource_references: Vec<ChatResourceReferenceInput>,
    #[serde(default)]
    pub also_send_to_channel: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleChatMessageCommand {
    pub scheduled_message_id: ChatScheduledMessageId,
    pub scheduled_for: UtcTimestamp,
    pub message: PostChatMessageCommand,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostChatMessageResult {
    pub message: ChatMessageRead,
    pub reply_thread_id: Option<ChatReplyThreadId>,
    pub assignment: Option<ChatWorkAssignmentRead>,
    pub assignment_input_queued: bool,
}

fn default_addressable() -> bool {
    true
}
