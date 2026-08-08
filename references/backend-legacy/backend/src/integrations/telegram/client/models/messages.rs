use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::errors::TelegramError;
use super::super::validation::validate_non_empty;
use super::chats::TelegramChatKind;
use makosh_communications_api::evidence::NewRawCommunicationRecord;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewTelegramMessage {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub chat_kind: TelegramChatKind,
    pub chat_title: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub text: String,
    pub import_batch_id: String,
    pub occurred_at: DateTime<Utc>,
    pub delivery_state: TelegramDeliveryState,
}

impl NewTelegramMessage {
    fn validate(&self) -> Result<(), TelegramError> {
        self.validate_common()?;
        validate_non_empty("text", &self.text)?;
        Ok(())
    }

    pub(in crate::integrations::telegram::client) fn validate_for_runtime(
        &self,
        runtime_kind: &str,
    ) -> Result<(), TelegramError> {
        if runtime_kind == "tdlib" {
            self.validate_common()
        } else {
            self.validate()
        }
    }

    fn validate_common(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("chat_title", &self.chat_title)?;
        validate_non_empty("sender_id", &self.sender_id)?;
        validate_non_empty("sender_display_name", &self.sender_display_name)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(in crate::integrations::telegram::client) fn source_fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.account_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_chat_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_message_id.as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramDeliveryState {
    Received,
    Queued,
    Sent,
    SendFailed,
    SendDryRun,
    SendBlocked,
}

impl TelegramDeliveryState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Queued => "queued",
            Self::Sent => "sent",
            Self::SendFailed => "send_failed",
            Self::SendDryRun => "send_dry_run",
            Self::SendBlocked => "send_blocked",
        }
    }

    pub fn as_message_delivery_state(self) -> &'static str {
        self.as_str()
    }
}

impl TryFrom<String> for TelegramDeliveryState {
    type Error = TelegramError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "received" => Ok(Self::Received),
            "queued" => Ok(Self::Queued),
            "sent" => Ok(Self::Sent),
            "send_failed" => Ok(Self::SendFailed),
            "send_dry_run" => Ok(Self::SendDryRun),
            "send_blocked" => Ok(Self::SendBlocked),
            _ => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram delivery_state `{value}`"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessageIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramObservedMessage {
    pub raw_record_id: String,
    pub message_id: String,
    pub raw: NewRawCommunicationRecord,
    pub telegram_chat_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramManualSendRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub text: String,
}

impl TelegramManualSendRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("text", &self.text)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramManualSendResponse {
    #[serde(skip_serializing)]
    pub raw: Option<NewRawCommunicationRecord>,
    pub raw_record_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub delivery_state: String,
    pub status: String,
    pub runtime_kind: String,
    pub rendered_preview_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramReplyRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub reply_to_provider_message_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramForwardRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub from_provider_chat_id: String,
    pub from_provider_message_id: String,
}

impl TelegramForwardRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("from_provider_chat_id", &self.from_provider_chat_id)?;
        validate_non_empty("from_provider_message_id", &self.from_provider_message_id)?;
        Ok(())
    }
}

impl TelegramReplyRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty(
            "reply_to_provider_message_id",
            &self.reply_to_provider_message_id,
        )?;
        validate_non_empty("text", &self.text)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: Option<String>,
    pub chat_title: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub delivery_state: String,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramAttachmentAnchor {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
}

// ---------------------------------------------------------------------------
// Message lifecycle — ADR-0091: versions, tombstones, provider-write commands
// ---------------------------------------------------------------------------

/// An append-only observed edit version of a Telegram message.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramMessageVersion {
    pub version_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: String,
    pub version_number: i32,
    pub body_text: Option<String>,
    pub edit_timestamp: DateTime<Utc>,
    pub source_event: Option<String>,
    pub raw_diff_payload: serde_json::Value,
    pub provenance: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Reason class for a tombstone per ADR-0091.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TombstoneReasonClass {
    DeletedByOwner,
    DeletedByCounterparty,
    DeletedByProvider,
    ModerationRemoved,
    AccountRemoved,
    RetentionPolicy,
    Unknown,
}

impl TombstoneReasonClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeletedByOwner => "deleted_by_owner",
            Self::DeletedByCounterparty => "deleted_by_counterparty",
            Self::DeletedByProvider => "deleted_by_provider",
            Self::ModerationRemoved => "moderation_removed",
            Self::AccountRemoved => "account_removed",
            Self::RetentionPolicy => "retention_policy",
            Self::Unknown => "unknown",
        }
    }
}

impl TryFrom<&str> for TombstoneReasonClass {
    type Error = TelegramError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "deleted_by_owner" => Ok(Self::DeletedByOwner),
            "deleted_by_counterparty" => Ok(Self::DeletedByCounterparty),
            "deleted_by_provider" => Ok(Self::DeletedByProvider),
            "moderation_removed" => Ok(Self::ModerationRemoved),
            "account_removed" => Ok(Self::AccountRemoved),
            "retention_policy" => Ok(Self::RetentionPolicy),
            "unknown" => Ok(Self::Unknown),
            other => Err(TelegramError::InvalidRequest(format!(
                "unsupported tombstone reason class `{other}`"
            ))),
        }
    }
}

/// Actor class for tombstone provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TombstoneActorClass {
    Owner,
    Provider,
    Automation,
    System,
    Unknown,
}

impl TombstoneActorClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Provider => "provider",
            Self::Automation => "automation",
            Self::System => "system",
            Self::Unknown => "unknown",
        }
    }
}

/// A local visibility and delete evidence record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramMessageTombstone {
    pub tombstone_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: String,
    pub reason_class: String,
    pub actor_class: String,
    pub observed_at: DateTime<Utc>,
    pub source_event: Option<String>,
    pub is_provider_delete: bool,
    pub is_local_visible: bool,
    pub metadata: serde_json::Value,
    pub provenance: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Durable provider-write command kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramCommandKind {
    SendText,
    SendMedia,
    Edit,
    Delete,
    RestoreVisibility,
    MarkRead,
    MarkUnread,
    Pin,
    Unpin,
    Archive,
    Unarchive,
    Mute,
    Unmute,
    React,
    Unreact,
    Reply,
    Forward,
    Join,
    Leave,
    FolderAdd,
    FolderRemove,
    TopicCreate,
    TopicClose,
    TopicReopen,
    AdminAction,
}

impl TelegramCommandKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SendText => "send_text",
            Self::SendMedia => "send_media",
            Self::Edit => "edit",
            Self::Delete => "delete",
            Self::RestoreVisibility => "restore_visibility",
            Self::MarkRead => "mark_read",
            Self::MarkUnread => "mark_unread",
            Self::Pin => "pin",
            Self::Unpin => "unpin",
            Self::Archive => "archive",
            Self::Unarchive => "unarchive",
            Self::Mute => "mute",
            Self::Unmute => "unmute",
            Self::React => "react",
            Self::Unreact => "unreact",
            Self::Reply => "reply",
            Self::Forward => "forward",
            Self::Join => "join",
            Self::Leave => "leave",
            Self::FolderAdd => "folder_add",
            Self::FolderRemove => "folder_remove",
            Self::TopicCreate => "topic_create",
            Self::TopicClose => "topic_close",
            Self::TopicReopen => "topic_reopen",
            Self::AdminAction => "admin_action",
        }
    }
}

/// A durable provider-write command record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramProviderWriteCommand {
    pub command_id: String,
    pub account_id: String,
    pub command_kind: String,
    pub idempotency_key: String,
    pub provider_chat_id: String,
    pub provider_message_id: Option<String>,
    pub target_ref: serde_json::Value,
    pub payload: serde_json::Value,
    pub capability_state: String,
    pub action_class: String,
    pub confirmation_decision: String,
    pub status: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub lease_epoch: i64,
    pub last_error: Option<String>,
    pub result_payload: serde_json::Value,
    pub audit_metadata: serde_json::Value,
    pub actor_id: String,
    pub happened_at: DateTime<Utc>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub locked_at: Option<DateTime<Utc>>,
    pub locked_by: Option<String>,
    pub provider_observed_at: Option<DateTime<Utc>>,
    pub provider_state: serde_json::Value,
    pub reconciliation_status: String,
    pub reconciled_at: Option<DateTime<Utc>>,
    pub dead_lettered_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request payload for editing a Telegram message.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramEditRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub new_text: String,
}

impl TelegramEditRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        if self.new_text.is_empty() {
            return Err(TelegramError::InvalidRequest(
                "edit request new_text must not be empty".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Request payload for deleting or restoring visibility of a Telegram message.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramDeleteRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub reason_class: String,
    pub actor_class: String,
    pub is_provider_delete: bool,
}

impl TelegramDeleteRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        // Validate reason_class is parseable
        TombstoneReasonClass::try_from(self.reason_class.as_str())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramRestoreVisibilityRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub reason: String,
}

impl TelegramRestoreVisibilityRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramPinRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub is_pinned: bool,
}

impl TelegramPinRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        Ok(())
    }
}

/// Response after a lifecycle operation (edit, delete, restore).
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramLifecycleResponse {
    pub operation: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version_number: Option<i32>,
    pub tombstone_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessageVersionListResponse {
    pub message_id: String,
    pub versions: Vec<TelegramMessageVersion>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessageTombstoneListResponse {
    pub message_id: String,
    pub tombstones: Vec<TelegramMessageTombstone>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramCommandListResponse {
    pub items: Vec<TelegramProviderWriteCommand>,
}

// ---------------------------------------------------------------------------
// Reactions (ADR-0091)
// ---------------------------------------------------------------------------

/// A single reaction on a Telegram message.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramReaction {
    pub reaction_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: String,
    pub sender_id: String,
    pub sender_display_name: Option<String>,
    pub reaction_emoji: String,
    pub is_active: bool,
    pub observed_at: DateTime<Utc>,
    pub source_event: Option<String>,
    pub provider_actor_id: Option<String>,
    pub metadata: serde_json::Value,
    pub provenance: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to add or update a reaction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramReactionRequest {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub reaction_emoji: String,
    pub sender_id: String,
    pub sender_display_name: Option<String>,
    pub command_id: Option<String>,
}

impl TelegramReactionRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("reaction_emoji", &self.reaction_emoji)?;
        validate_non_empty("sender_id", &self.sender_id)?;
        Ok(())
    }
}

/// Response for a single reaction operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramReactionResponse {
    pub reaction_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub reaction_emoji: String,
    pub is_active: bool,
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

/// Aggregate reaction summary for a message.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramReactionSummary {
    pub message_id: String,
    pub total_reactions: i64,
    pub active_reactions: i64,
    pub reactions: Vec<TelegramReactionGroup>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramReactionGroup {
    pub reaction_emoji: String,
    pub count: i64,
    pub senders: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramReactionListResponse {
    pub message_id: String,
    pub reactions: Vec<TelegramReaction>,
    pub summary: TelegramReactionSummary,
}
