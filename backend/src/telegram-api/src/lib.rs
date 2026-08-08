//! Telegram provider contract. No business-domain types or provider SDK types belong here.

use serde::{Deserialize, Serialize};

pub mod client_contract;
pub mod client_wire;
#[allow(clippy::large_enum_variant)]
pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.telegram.v1.rs"));
}

pub const PACKAGE: &str = "makosh-telegram-api";
pub const MAX_ID_LEN: usize = 256;
pub const MAX_TEXT_BYTES: usize = 256 * 1024;
pub const DEFAULT_PAGE_SIZE: u32 = 100;
pub const MAX_PAGE_SIZE: u32 = 5_000;

pub type TelegramAccountId = String;
pub type TelegramOperationId = String;
pub type TelegramSetupId = String;
pub type TelegramRuntimeReconfigurationId = String;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramCredentialPurpose {
    ApiHash,
    SessionEncryptionKey,
}

impl TelegramCredentialPurpose {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApiHash => "telegram_api_hash",
            Self::SessionEncryptionKey => "telegram_session_encryption_key",
        }
    }

    pub const fn is_session_store_key(self) -> bool {
        matches!(self, Self::SessionEncryptionKey)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramCredentialBinding {
    pub purpose: TelegramCredentialPurpose,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramAccountState {
    Provisioning,
    Ready,
    Degraded,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramRuntimeState {
    Stopped,
    Starting,
    Running,
    Degraded,
    Blocked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramQrLoginState {
    Preparing,
    WaitingQrScan,
    WaitingPassword,
    Ready,
    Expired,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramQrLoginSession {
    pub setup_id: TelegramSetupId,
    pub account_id: TelegramAccountId,
    pub state: TelegramQrLoginState,
    pub qr_link: Option<String>,
    pub password_attempts: u8,
    pub expires_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramAuthorizationStatus {
    pub state: String,
    pub qr_link: Option<String>,
    pub password_hint: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramAccountSetup {
    pub account_id: TelegramAccountId,
    pub display_name: String,
    pub external_account_id: String,
    pub credentials: Vec<TelegramCredentialBinding>,
    pub qr_authorized: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramAccount {
    pub account_id: TelegramAccountId,
    pub display_name: String,
    pub external_account_id: String,
    pub state: TelegramAccountState,
    pub runtime_state: TelegramRuntimeState,
    pub runtime_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramRuntimeLeaseState {
    Active,
    Revoked,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramRuntimeLease {
    pub account_id: TelegramAccountId,
    pub topology: String,
    pub holder: String,
    pub epoch: u64,
    pub state: TelegramRuntimeLeaseState,
    pub expires_at_unix_seconds: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramRuntimeReconfigurationState {
    Accepted,
    Applying,
    Completed,
    Failed,
}

impl TelegramRuntimeReconfigurationState {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramRuntimeReconfiguration {
    pub reconfiguration_id: TelegramRuntimeReconfigurationId,
    pub account_id: TelegramAccountId,
    pub expected_runtime_epoch: u64,
    pub target_runtime_epoch: u64,
    pub state: TelegramRuntimeReconfigurationState,
    pub sanitized_reason_code: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramRuntimeReconfigurationRequest {
    Begin {
        reconfiguration_id: TelegramRuntimeReconfigurationId,
        account_id: TelegramAccountId,
        expected_runtime_epoch: u64,
    },
    Status {
        reconfiguration_id: TelegramRuntimeReconfigurationId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramChat {
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub kind: TelegramChatKind,
    pub title: String,
    pub username: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramChatAvatar {
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub provider_file_id: Option<String>,
    pub provider_unique_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramTopic {
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub provider_topic_id: String,
    pub title: String,
    pub icon_emoji: Option<String>,
    pub is_pinned: bool,
    pub is_closed: bool,
    pub unread_count: u32,
    pub last_message_at_unix_seconds: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramTypingState {
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub provider_thread_id: Option<String>,
    pub sender_id: String,
    pub action: String,
    pub is_active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramChatPosition {
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub list_kind: String,
    pub provider_folder_id: Option<i64>,
    pub order: i64,
    pub is_pinned: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramChatFolder {
    pub account_id: TelegramAccountId,
    pub provider_folder_id: i64,
    pub title: String,
    pub icon_name: Option<String>,
    pub color_id: Option<i64>,
    pub pinned_chat_ids: Vec<String>,
    pub included_chat_ids: Vec<String>,
    pub excluded_chat_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramChatKind {
    Private,
    Group,
    Channel,
    Bot,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramMessageObservation {
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    #[serde(default)]
    pub provider_topic_id: Option<String>,
    pub sender_id: String,
    pub sender_display_name: Option<String>,
    pub is_outgoing: bool,
    pub text: Option<String>,
    pub media: Option<TelegramMessageMedia>,
    pub references: TelegramMessageReferences,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct TelegramMessageReferences {
    pub reply_to: Option<TelegramReplyReference>,
    pub forward_origin: Option<TelegramForwardOrigin>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramReplyReference {
    pub provider_chat_id: String,
    pub provider_message_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramForwardOrigin {
    #[serde(default)]
    pub provider_chat_id: Option<String>,
    #[serde(default)]
    pub provider_message_id: Option<String>,
    pub provider_sender_id: Option<String>,
    pub sender_name: Option<String>,
    pub observed_at_unix_seconds: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramMessageMedia {
    pub kind: TelegramMediaKind,
    pub provider_file_id: Option<String>,
    pub caption: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramMessageProjection {
    pub message_id: String,
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    #[serde(default)]
    pub provider_topic_id: Option<String>,
    pub sender_id: String,
    pub sender_display_name: Option<String>,
    pub text: Option<String>,
    pub media: Option<TelegramMessageMedia>,
    pub references: TelegramMessageReferences,
    pub observed_at_unix_seconds: i64,
    pub delivery_state: TelegramDeliveryState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramMediaKind {
    Photo,
    Video,
    Audio,
    Document,
    Animation,
    VoiceNote,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramFileSnapshot {
    pub account_id: TelegramAccountId,
    pub provider_file_id: String,
    pub provider_unique_id: Option<String>,
    pub media_kind: Option<TelegramMediaKind>,
    pub size_bytes: Option<u64>,
    pub expected_size_bytes: Option<u64>,
    pub downloaded_size_bytes: Option<u64>,
    pub is_downloading: bool,
    pub is_downloaded: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramDownloadFile {
    pub operation_id: TelegramOperationId,
    pub account_id: TelegramAccountId,
    pub provider_file_id: String,
    pub priority: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramSendMedia {
    pub operation_id: TelegramOperationId,
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub media_kind: TelegramMediaKind,
    pub blob: TelegramBlobIntentV1,
    pub caption: Option<String>,
    pub filename: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramBlobIntentV1 {
    pub blob_ref: String,
    pub reference_id: Vec<u8>,
    pub declared_size: u64,
    pub backup_class: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramParticipant {
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub provider_member_id: String,
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub role: String,
    pub status: String,
    pub is_admin: bool,
    pub is_owner: bool,
    pub permissions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramParticipantPage {
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub filter: TelegramParticipantFilter,
    pub items: Vec<TelegramParticipant>,
    pub next_offset: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramParticipantFilter {
    Recent,
    Administrators,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramRealtimeFrame {
    pub account_id: TelegramAccountId,
    pub sequence: u64,
    pub provider_cursor: Option<String>,
    pub event: TelegramProviderEvent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramDeliveryState {
    Received,
    Queued,
    Sent,
    SendFailed,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramProviderEvent {
    MessageCreated(TelegramMessageObservation),
    MessageEdited {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        text: Option<String>,
        observed_at_unix_seconds: i64,
    },
    MessageDeleted {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        is_permanent: bool,
    },
    MessageSendFailed {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        old_provider_message_id: String,
        error_code: Option<i64>,
    },
    MessageSendSucceeded {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        old_provider_message_id: String,
        provider_message_id: String,
    },
    MessagePinned {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        is_pinned: bool,
    },
    ReactionChanged {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        emoji: Option<String>,
        is_active: bool,
    },
    ReactionsObserved {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        reactions: Vec<TelegramReactionObservation>,
    },
    ChatUnreadChanged {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        unread_count: Option<i64>,
        unread_mention_count: Option<i64>,
        last_read_inbox_message_id: Option<String>,
    },
    ChatMarkedUnreadChanged {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        is_marked_as_unread: bool,
    },
    TypingChanged(TelegramTypingState),
    TopicChanged(TelegramTopic),
    ChatPositionChanged(TelegramChatPosition),
    ChatFoldersChanged {
        account_id: TelegramAccountId,
        folders: Vec<TelegramChatFolder>,
    },
    ChatNotificationChanged {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        use_default_mute_for: bool,
        mute_for_seconds: i64,
    },
    ChatAvatarChanged(TelegramChatAvatar),
    ParticipantChanged(TelegramParticipant),
    FileChanged(TelegramFileSnapshot),
}

pub fn provider_event_account_id(event: &TelegramProviderEvent) -> &str {
    match event {
        TelegramProviderEvent::MessageCreated(observation) => &observation.account_id,
        TelegramProviderEvent::MessageEdited { account_id, .. }
        | TelegramProviderEvent::MessageDeleted { account_id, .. }
        | TelegramProviderEvent::MessageSendFailed { account_id, .. }
        | TelegramProviderEvent::MessageSendSucceeded { account_id, .. }
        | TelegramProviderEvent::MessagePinned { account_id, .. }
        | TelegramProviderEvent::ReactionChanged { account_id, .. }
        | TelegramProviderEvent::ReactionsObserved { account_id, .. }
        | TelegramProviderEvent::ChatUnreadChanged { account_id, .. }
        | TelegramProviderEvent::ChatMarkedUnreadChanged { account_id, .. }
        | TelegramProviderEvent::ChatFoldersChanged { account_id, .. }
        | TelegramProviderEvent::ChatNotificationChanged { account_id, .. } => account_id,
        TelegramProviderEvent::TypingChanged(state) => &state.account_id,
        TelegramProviderEvent::ChatAvatarChanged(avatar) => &avatar.account_id,
        TelegramProviderEvent::ParticipantChanged(participant) => &participant.account_id,
        TelegramProviderEvent::TopicChanged(topic) => &topic.account_id,
        TelegramProviderEvent::ChatPositionChanged(position) => &position.account_id,
        TelegramProviderEvent::FileChanged(file) => &file.account_id,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramMessageMutation {
    Edit {
        text: Option<String>,
        observed_at_unix_seconds: i64,
    },
    Delete {
        is_permanent: bool,
    },
    Pin {
        is_pinned: bool,
    },
    Reaction {
        emoji: Option<String>,
        is_active: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramMessageVersionSource {
    Provider,
    Owner,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramMessageVersion {
    pub version_id: String,
    pub message_id: String,
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub version_number: u32,
    pub body_text: Option<String>,
    pub observed_at_unix_seconds: i64,
    pub source: TelegramMessageVersionSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramTombstoneReason {
    ProviderDeleted,
    OwnerDeleted,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramMessageTombstone {
    pub tombstone_id: String,
    pub message_id: String,
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub reason: TelegramTombstoneReason,
    pub observed_at_unix_seconds: i64,
    pub is_provider_delete: bool,
    pub is_locally_visible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramReactionObservation {
    pub sender_id: String,
    pub emoji: String,
    pub is_outgoing: bool,
    pub is_active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramReactionSummary {
    pub emoji: String,
    pub count: u32,
    pub is_active: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramAttachmentDownloadState {
    Pending,
    Downloading,
    Downloaded,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramAttachmentProjection {
    pub attachment_id: String,
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub provider_file_id: String,
    pub state: TelegramAttachmentDownloadState,
    pub size_bytes: Option<u64>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub blob_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct TelegramChatStateProjection {
    pub unread_count: Option<i64>,
    pub unread_mention_count: Option<i64>,
    pub last_read_inbox_message_id: Option<String>,
    pub is_marked_as_unread: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct TelegramChatOperationalState {
    pub is_archived: bool,
    pub is_pinned: bool,
    pub is_muted: bool,
    pub mute_for_seconds: i64,
    pub is_marked_as_unread: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramHistorySyncMode {
    Latest,
    Older,
    Full,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramHistoryPage {
    pub items: Vec<TelegramMessageObservation>,
    pub next_from_message_id: Option<i64>,
    pub has_more: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramSendMessage {
    pub operation_id: TelegramOperationId,
    pub account_id: TelegramAccountId,
    pub provider_chat_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramProviderCommand {
    SendText(TelegramSendMessage),
    SendMedia(TelegramSendMedia),
    DownloadFile(TelegramDownloadFile),
    Reply {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        reply_to_provider_message_id: String,
        text: String,
    },
    Forward {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        from_provider_chat_id: String,
        from_provider_message_id: String,
    },
    Edit {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        text: String,
    },
    Delete {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        revoke: bool,
    },
    RestoreVisibility {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        reason: String,
    },
    Reaction {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        emoji: String,
        active: bool,
    },
    Pin {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        active: bool,
    },
    MarkUnread {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        unread: bool,
        read_through_provider_message_id: Option<String>,
    },
    Archive {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        archived: bool,
    },
    Mute {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        muted: bool,
    },
    Join {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
    },
    Leave {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
    },
    AddChatToFolder {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_folder_id: i64,
    },
    RemoveChatFromFolder {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_folder_id: i64,
    },
    ReassignChatFolders {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        target_provider_folder_ids: Vec<i64>,
    },
    SearchMessages {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: Option<String>,
        query: String,
        limit: u32,
    },
    ListParticipants {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        filter: TelegramParticipantFilter,
        offset: u32,
        limit: u32,
    },
    ListTopics {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        limit: u32,
    },
    CreateTopic {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        title: String,
    },
    SetTopicClosed {
        operation_id: TelegramOperationId,
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_topic_id: String,
        is_closed: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramProviderQuery {
    LoadChats {
        account_id: TelegramAccountId,
        limit: u32,
    },
    Chat {
        account_id: TelegramAccountId,
        provider_chat_id: String,
    },
    ChatAvatar {
        account_id: TelegramAccountId,
        provider_chat_id: String,
    },
    LoadHistory {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        from_message_id: Option<i64>,
        mode: TelegramHistorySyncMode,
        limit: u32,
    },
    CachedChats {
        account_id: TelegramAccountId,
        limit: u32,
    },
    SearchChats {
        account_id: TelegramAccountId,
        query: String,
        limit: u32,
    },
    CachedMessages {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        limit: u32,
    },
    MessageById {
        account_id: TelegramAccountId,
        message_id: String,
    },
    RecentMessages {
        account_id: TelegramAccountId,
        provider_chat_id: Option<String>,
        limit: u32,
    },
    MessagesByIds {
        account_id: TelegramAccountId,
        message_ids: Vec<String>,
    },
    MessageVersions {
        account_id: TelegramAccountId,
        message_id: String,
    },
    MessageTombstones {
        account_id: TelegramAccountId,
        message_id: String,
    },
    MessageMutations {
        account_id: TelegramAccountId,
        message_id: String,
    },
    MessageReferences {
        account_id: TelegramAccountId,
        message_id: String,
    },
    ReplyChain {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        limit: u32,
    },
    ForwardChain {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
        limit: u32,
    },
    Attachment {
        account_id: TelegramAccountId,
        attachment_id: String,
    },
    AttachmentForMessage {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
    },
    File {
        account_id: TelegramAccountId,
        provider_file_id: String,
    },
    ChatState {
        account_id: TelegramAccountId,
        provider_chat_id: String,
    },
    ChatPositions {
        account_id: TelegramAccountId,
        provider_chat_id: String,
    },
    ChatOperationalState {
        account_id: TelegramAccountId,
        provider_chat_id: String,
    },
    PinnedMessages {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        limit: u32,
    },
    SearchMessages {
        account_id: TelegramAccountId,
        provider_chat_id: Option<String>,
        query: String,
        limit: u32,
    },
    ListParticipants {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        filter: TelegramParticipantFilter,
        offset: u32,
        limit: u32,
    },
    BasicGroupParticipants {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        basic_group_id: i64,
    },
    ListTopics {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        limit: u32,
    },
    Topic {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_topic_id: String,
    },
    TopicMessageIds {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_topic_id: String,
        limit: u32,
    },
    SearchTopics {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        query: String,
        limit: u32,
    },
    Reactions {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
    },
    ReactionSummary {
        account_id: TelegramAccountId,
        provider_chat_id: String,
        provider_message_id: String,
    },
    ChatFolder {
        account_id: TelegramAccountId,
        provider_folder_id: i64,
    },
    ChatFolders {
        account_id: TelegramAccountId,
        provider_folder_ids: Vec<i64>,
    },
    Operations {
        account_id: TelegramAccountId,
        limit: u32,
    },
    Commands {
        account_id: TelegramAccountId,
        provider_chat_id: Option<String>,
        provider_message_id: Option<String>,
        command_kinds: Vec<String>,
        limit: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramProviderQueryResponse {
    Chats(Vec<TelegramChat>),
    Chat(Option<TelegramChat>),
    ChatAvatar(Option<TelegramChatAvatar>),
    History(Vec<TelegramMessageObservation>),
    HistoryPage(TelegramHistoryPage),
    CachedMessages(Vec<TelegramMessageProjection>),
    MessageVersions(Vec<TelegramMessageVersion>),
    MessageTombstones(Vec<TelegramMessageTombstone>),
    MessageMutations(Vec<TelegramMessageMutation>),
    MessageReferences(Option<TelegramMessageReferences>),
    ReplyChain(Vec<TelegramMessageProjection>),
    ForwardChain(Vec<TelegramMessageProjection>),
    Attachment(Option<TelegramAttachmentProjection>),
    File(Option<TelegramFileSnapshot>),
    ChatState(Option<TelegramChatStateProjection>),
    ChatPositions(Vec<TelegramChatPosition>),
    ChatOperationalState(Option<TelegramChatOperationalState>),
    Participants(TelegramParticipantPage),
    Topics(Vec<TelegramTopic>),
    Topic(Option<TelegramTopic>),
    TopicMessageIds(Vec<String>),
    Reactions(Vec<TelegramReactionObservation>),
    ReactionSummary(Vec<TelegramReactionSummary>),
    ChatFolders(Vec<TelegramChatFolder>),
    Operations(Vec<TelegramOperation>),
    Commands(Vec<TelegramCommandRecord>),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramCommandRecord {
    pub operation: TelegramOperation,
    pub command: TelegramProviderCommand,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramClientRequest {
    Command(TelegramProviderCommand),
    Query(TelegramProviderQuery),
    ProvisionAccount {
        setup: TelegramAccountSetup,
    },
    RetryCommand {
        operation_id: TelegramOperationId,
        now_unix_seconds: u64,
        next_attempt_at_unix_seconds: u64,
    },
    ListAccounts,
    GetAccount {
        account_id: TelegramAccountId,
    },
    RetireAccount {
        account_id: TelegramAccountId,
    },
    AuthorizationStatus,
    SubmitAuthorizationPassword {
        password: String,
    },
    Reconfiguration(TelegramRuntimeReconfigurationRequest),
    Replay {
        account_id: TelegramAccountId,
        after_sequence: u64,
        limit: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramClientResponse {
    Accepted { operation_id: TelegramOperationId },
    Operation(TelegramOperation),
    Accounts(Vec<TelegramAccount>),
    Query(TelegramProviderQueryResponse),
    AuthorizationStatus(TelegramAuthorizationStatus),
    AuthorizationPasswordAccepted,
    Account(TelegramAccount),
    Reconfiguration(TelegramRuntimeReconfiguration),
    Realtime(TelegramRealtimeReplayPage),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramRealtimeReplayPage {
    pub frames: Vec<TelegramRealtimeFrame>,
    pub earliest_available_sequence: u64,
    pub latest_sequence: u64,
    pub next_after_sequence: u64,
    pub reset_required: bool,
}

pub fn validate_provider_query(query: &TelegramProviderQuery) -> Result<(), TelegramContractError> {
    let (account_id, limit) = match query {
        TelegramProviderQuery::LoadChats { account_id, limit }
        | TelegramProviderQuery::CachedChats { account_id, limit }
        | TelegramProviderQuery::SearchChats {
            account_id, limit, ..
        }
        | TelegramProviderQuery::ListTopics {
            account_id, limit, ..
        }
        | TelegramProviderQuery::ListParticipants {
            account_id, limit, ..
        }
        | TelegramProviderQuery::Operations { account_id, limit }
        | TelegramProviderQuery::Commands {
            account_id, limit, ..
        }
        | TelegramProviderQuery::TopicMessageIds {
            account_id, limit, ..
        } => (account_id, *limit),
        TelegramProviderQuery::LoadHistory {
            account_id, limit, ..
        }
        | TelegramProviderQuery::CachedMessages {
            account_id, limit, ..
        } => (account_id, *limit),
        TelegramProviderQuery::RecentMessages {
            account_id, limit, ..
        }
        | TelegramProviderQuery::PinnedMessages {
            account_id, limit, ..
        }
        | TelegramProviderQuery::SearchTopics {
            account_id, limit, ..
        }
        | TelegramProviderQuery::ReplyChain {
            account_id, limit, ..
        }
        | TelegramProviderQuery::ForwardChain {
            account_id, limit, ..
        } => (account_id, *limit),
        TelegramProviderQuery::MessageById { account_id, .. }
        | TelegramProviderQuery::MessagesByIds { account_id, .. }
        | TelegramProviderQuery::MessageVersions { account_id, .. }
        | TelegramProviderQuery::MessageTombstones { account_id, .. }
        | TelegramProviderQuery::MessageMutations { account_id, .. }
        | TelegramProviderQuery::MessageReferences { account_id, .. }
        | TelegramProviderQuery::Attachment { account_id, .. }
        | TelegramProviderQuery::AttachmentForMessage { account_id, .. }
        | TelegramProviderQuery::File { account_id, .. }
        | TelegramProviderQuery::ChatState { account_id, .. }
        | TelegramProviderQuery::ChatPositions { account_id, .. }
        | TelegramProviderQuery::ChatOperationalState { account_id, .. }
        | TelegramProviderQuery::Chat { account_id, .. }
        | TelegramProviderQuery::ChatAvatar { account_id, .. }
        | TelegramProviderQuery::Topic { account_id, .. }
        | TelegramProviderQuery::BasicGroupParticipants { account_id, .. }
        | TelegramProviderQuery::ChatFolder { account_id, .. }
        | TelegramProviderQuery::ChatFolders { account_id, .. } => (account_id, 1),
        TelegramProviderQuery::SearchMessages {
            account_id, limit, ..
        } => (account_id, *limit),
        TelegramProviderQuery::Reactions { account_id, .. }
        | TelegramProviderQuery::ReactionSummary { account_id, .. } => (account_id, 1),
    };
    validate_id(account_id)?;
    if !matches!(
        query,
        TelegramProviderQuery::Reactions { .. } | TelegramProviderQuery::ReactionSummary { .. }
    ) {
        validate_page_size(limit)?;
    }
    match query {
        TelegramProviderQuery::LoadHistory {
            provider_chat_id,
            from_message_id,
            mode,
            ..
        } => {
            validate_id(provider_chat_id)?;
            if from_message_id.is_some_and(|message_id| message_id <= 0) {
                return Err(TelegramContractError::InvalidPageSize);
            }
            if matches!(mode, TelegramHistorySyncMode::Older) && from_message_id.is_none() {
                return Err(TelegramContractError::EmptyField);
            }
            Ok(())
        }
        TelegramProviderQuery::CachedMessages {
            provider_chat_id, ..
        }
        | TelegramProviderQuery::RecentMessages {
            provider_chat_id: Some(provider_chat_id),
            ..
        }
        | TelegramProviderQuery::ListParticipants {
            provider_chat_id, ..
        }
        | TelegramProviderQuery::ListTopics {
            provider_chat_id, ..
        }
        | TelegramProviderQuery::PinnedMessages {
            provider_chat_id, ..
        }
        | TelegramProviderQuery::Reactions {
            provider_chat_id, ..
        }
        | TelegramProviderQuery::ReactionSummary {
            provider_chat_id, ..
        } => validate_id(provider_chat_id),
        TelegramProviderQuery::Chat {
            provider_chat_id, ..
        } => validate_id(provider_chat_id),
        TelegramProviderQuery::ChatAvatar {
            provider_chat_id, ..
        } => validate_id(provider_chat_id),
        TelegramProviderQuery::Topic {
            provider_chat_id,
            provider_topic_id,
            ..
        } => {
            validate_id(provider_chat_id)?;
            validate_id(provider_topic_id)
        }
        TelegramProviderQuery::TopicMessageIds {
            provider_chat_id,
            provider_topic_id,
            ..
        } => {
            validate_id(provider_chat_id)?;
            validate_id(provider_topic_id)
        }
        TelegramProviderQuery::RecentMessages {
            provider_chat_id: None,
            ..
        } => Ok(()),
        TelegramProviderQuery::MessageById { message_id, .. } => validate_id(message_id),
        TelegramProviderQuery::MessagesByIds { message_ids, .. } => {
            if message_ids.is_empty() {
                return Err(TelegramContractError::EmptyField);
            }
            for message_id in message_ids {
                validate_id(message_id)?;
            }
            Ok(())
        }
        TelegramProviderQuery::MessageVersions { message_id, .. }
        | TelegramProviderQuery::MessageTombstones { message_id, .. }
        | TelegramProviderQuery::MessageMutations { message_id, .. }
        | TelegramProviderQuery::MessageReferences { message_id, .. } => validate_id(message_id),
        TelegramProviderQuery::ReplyChain {
            provider_chat_id,
            provider_message_id,
            ..
        } => {
            validate_id(provider_chat_id)?;
            validate_id(provider_message_id)
        }
        TelegramProviderQuery::ForwardChain {
            provider_chat_id,
            provider_message_id,
            ..
        } => {
            validate_id(provider_chat_id)?;
            validate_id(provider_message_id)
        }
        TelegramProviderQuery::Attachment { attachment_id, .. } => validate_id(attachment_id),
        TelegramProviderQuery::AttachmentForMessage {
            provider_chat_id,
            provider_message_id,
            ..
        } => {
            validate_id(provider_chat_id)?;
            validate_id(provider_message_id)
        }
        TelegramProviderQuery::File {
            provider_file_id, ..
        } => validate_id(provider_file_id),
        TelegramProviderQuery::SearchMessages {
            provider_chat_id,
            query,
            ..
        } => {
            if let Some(provider_chat_id) = provider_chat_id {
                validate_id(provider_chat_id)?;
            }
            validate_text(query)
        }
        TelegramProviderQuery::SearchChats { query, .. } => validate_text(query),
        TelegramProviderQuery::SearchTopics { query, .. } => validate_text(query),
        TelegramProviderQuery::ChatState {
            provider_chat_id, ..
        } => validate_id(provider_chat_id),
        TelegramProviderQuery::ChatPositions {
            provider_chat_id, ..
        } => validate_id(provider_chat_id),
        TelegramProviderQuery::ChatOperationalState {
            provider_chat_id, ..
        } => validate_id(provider_chat_id),
        TelegramProviderQuery::BasicGroupParticipants {
            provider_chat_id,
            basic_group_id,
            ..
        } => {
            validate_id(provider_chat_id)?;
            (*basic_group_id > 0)
                .then_some(())
                .ok_or(TelegramContractError::InvalidPageSize)
        }
        TelegramProviderQuery::ChatFolder {
            provider_folder_id, ..
        } => (*provider_folder_id > 0)
            .then_some(())
            .ok_or(TelegramContractError::InvalidPageSize),
        TelegramProviderQuery::ChatFolders {
            provider_folder_ids,
            ..
        } => provider_folder_ids
            .iter()
            .all(|provider_folder_id| *provider_folder_id > 0)
            .then_some(())
            .ok_or(TelegramContractError::InvalidPageSize),
        TelegramProviderQuery::LoadChats { .. }
        | TelegramProviderQuery::CachedChats { .. }
        | TelegramProviderQuery::Operations { .. } => Ok(()),
        TelegramProviderQuery::Commands {
            provider_chat_id,
            provider_message_id,
            command_kinds,
            ..
        } => {
            if let Some(value) = provider_chat_id {
                validate_id(value)?;
            }
            if let Some(value) = provider_message_id {
                validate_id(value)?;
            }
            for kind in command_kinds {
                validate_id(kind)?;
            }
            Ok(())
        }
    }
}

pub fn provider_query_account_id(query: &TelegramProviderQuery) -> &str {
    match query {
        TelegramProviderQuery::LoadChats { account_id, .. }
        | TelegramProviderQuery::Chat { account_id, .. }
        | TelegramProviderQuery::ChatAvatar { account_id, .. }
        | TelegramProviderQuery::LoadHistory { account_id, .. }
        | TelegramProviderQuery::CachedChats { account_id, .. }
        | TelegramProviderQuery::Operations { account_id, .. }
        | TelegramProviderQuery::SearchChats { account_id, .. }
        | TelegramProviderQuery::CachedMessages { account_id, .. }
        | TelegramProviderQuery::MessageById { account_id, .. }
        | TelegramProviderQuery::RecentMessages { account_id, .. }
        | TelegramProviderQuery::MessagesByIds { account_id, .. }
        | TelegramProviderQuery::MessageVersions { account_id, .. }
        | TelegramProviderQuery::MessageTombstones { account_id, .. }
        | TelegramProviderQuery::MessageMutations { account_id, .. }
        | TelegramProviderQuery::MessageReferences { account_id, .. }
        | TelegramProviderQuery::ReplyChain { account_id, .. }
        | TelegramProviderQuery::ForwardChain { account_id, .. }
        | TelegramProviderQuery::Attachment { account_id, .. }
        | TelegramProviderQuery::AttachmentForMessage { account_id, .. }
        | TelegramProviderQuery::File { account_id, .. }
        | TelegramProviderQuery::ChatState { account_id, .. }
        | TelegramProviderQuery::PinnedMessages { account_id, .. }
        | TelegramProviderQuery::SearchMessages { account_id, .. }
        | TelegramProviderQuery::ListParticipants { account_id, .. }
        | TelegramProviderQuery::Topic { account_id, .. }
        | TelegramProviderQuery::TopicMessageIds { account_id, .. }
        | TelegramProviderQuery::BasicGroupParticipants { account_id, .. }
        | TelegramProviderQuery::ListTopics { account_id, .. }
        | TelegramProviderQuery::SearchTopics { account_id, .. }
        | TelegramProviderQuery::Reactions { account_id, .. }
        | TelegramProviderQuery::ReactionSummary { account_id, .. }
        | TelegramProviderQuery::ChatPositions { account_id, .. }
        | TelegramProviderQuery::ChatOperationalState { account_id, .. }
        | TelegramProviderQuery::ChatFolder { account_id, .. }
        | TelegramProviderQuery::ChatFolders { account_id, .. } => account_id,
        TelegramProviderQuery::Commands { account_id, .. } => account_id,
    }
}

pub fn validate_provider_command(
    command: &TelegramProviderCommand,
) -> Result<(), TelegramContractError> {
    let (operation_id, account_id, chat_id) = match command {
        TelegramProviderCommand::SendText(command) => (
            &command.operation_id,
            &command.account_id,
            &command.provider_chat_id,
        ),
        TelegramProviderCommand::SendMedia(command) => (
            &command.operation_id,
            &command.account_id,
            &command.provider_chat_id,
        ),
        TelegramProviderCommand::DownloadFile(command) => (
            &command.operation_id,
            &command.account_id,
            &command.provider_file_id,
        ),
        TelegramProviderCommand::Reply {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::Forward {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::Edit {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::Delete {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::RestoreVisibility {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::Reaction {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::Pin {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::MarkUnread {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::Archive {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::Mute {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::Join {
            operation_id,
            account_id,
            provider_chat_id,
        }
        | TelegramProviderCommand::Leave {
            operation_id,
            account_id,
            provider_chat_id,
        }
        | TelegramProviderCommand::AddChatToFolder {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::RemoveChatFromFolder {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::ReassignChatFolders {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::SearchMessages {
            operation_id,
            account_id,
            provider_chat_id: Some(provider_chat_id),
            ..
        } => (operation_id, account_id, provider_chat_id),
        TelegramProviderCommand::SearchMessages {
            operation_id,
            account_id,
            provider_chat_id: None,
            ..
        } => (operation_id, account_id, account_id),
        TelegramProviderCommand::ListParticipants {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        } => (operation_id, account_id, provider_chat_id),
        TelegramProviderCommand::ListTopics {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::CreateTopic {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        }
        | TelegramProviderCommand::SetTopicClosed {
            operation_id,
            account_id,
            provider_chat_id,
            ..
        } => (operation_id, account_id, provider_chat_id),
    };
    validate_id(operation_id)?;
    validate_id(account_id)?;
    validate_id(chat_id)?;
    match command {
        TelegramProviderCommand::SendText(command) => validate_text(&command.text),
        TelegramProviderCommand::SendMedia(command) => {
            validate_id(&command.blob.blob_ref)?;
            if command.blob.reference_id.len() != 16
                || command.blob.reference_id.iter().all(|byte| *byte == 0)
                || command.blob.declared_size == 0
                || !(1..=3).contains(&command.blob.backup_class)
            {
                return Err(TelegramContractError::InvalidTransition);
            }
            if let Some(caption) = &command.caption {
                validate_text(caption)?;
            }
            if let Some(filename) = &command.filename {
                validate_id(filename)?;
            }
            Ok(())
        }
        TelegramProviderCommand::DownloadFile(command) => {
            if !(1..=32).contains(&command.priority) {
                return Err(TelegramContractError::InvalidPageSize);
            }
            Ok(())
        }
        TelegramProviderCommand::Reply {
            text,
            reply_to_provider_message_id,
            ..
        } => {
            validate_text(text)?;
            validate_id(reply_to_provider_message_id)
        }
        TelegramProviderCommand::Forward {
            from_provider_chat_id,
            from_provider_message_id,
            ..
        } => {
            validate_id(from_provider_chat_id)?;
            validate_id(from_provider_message_id)
        }
        TelegramProviderCommand::Edit {
            text,
            provider_message_id,
            ..
        } => {
            validate_text(text)?;
            validate_id(provider_message_id)
        }
        TelegramProviderCommand::Delete {
            provider_message_id,
            ..
        }
        | TelegramProviderCommand::Pin {
            provider_message_id,
            ..
        } => validate_id(provider_message_id),
        TelegramProviderCommand::RestoreVisibility {
            provider_message_id,
            reason,
            ..
        } => {
            validate_id(provider_message_id)?;
            validate_text(reason)
        }
        TelegramProviderCommand::Reaction {
            provider_message_id,
            emoji,
            ..
        } => {
            validate_id(provider_message_id)?;
            validate_text(emoji)
        }
        TelegramProviderCommand::SearchMessages { query, limit, .. } => {
            validate_text(query)?;
            validate_page_size(*limit)
        }
        TelegramProviderCommand::ListParticipants { limit, .. } => validate_page_size(*limit),
        TelegramProviderCommand::ListTopics { limit, .. } => validate_page_size(*limit),
        TelegramProviderCommand::CreateTopic { title, .. } => validate_text(title),
        TelegramProviderCommand::SetTopicClosed {
            provider_topic_id, ..
        } => validate_id(provider_topic_id),
        TelegramProviderCommand::ReassignChatFolders {
            target_provider_folder_ids,
            ..
        } => {
            if target_provider_folder_ids.is_empty()
                || target_provider_folder_ids.len() > 64
                || target_provider_folder_ids
                    .iter()
                    .any(|folder_id| *folder_id <= 0)
            {
                return Err(TelegramContractError::InvalidTransition);
            }
            let unique = target_provider_folder_ids
                .iter()
                .collect::<std::collections::BTreeSet<_>>();
            if unique.len() != target_provider_folder_ids.len() {
                return Err(TelegramContractError::InvalidTransition);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn provider_command_account_id(command: &TelegramProviderCommand) -> &str {
    match command {
        TelegramProviderCommand::SendText(command) => &command.account_id,
        TelegramProviderCommand::SendMedia(command) => &command.account_id,
        TelegramProviderCommand::DownloadFile(command) => &command.account_id,
        TelegramProviderCommand::Reply { account_id, .. }
        | TelegramProviderCommand::Forward { account_id, .. }
        | TelegramProviderCommand::Edit { account_id, .. }
        | TelegramProviderCommand::Delete { account_id, .. }
        | TelegramProviderCommand::RestoreVisibility { account_id, .. }
        | TelegramProviderCommand::Reaction { account_id, .. }
        | TelegramProviderCommand::Pin { account_id, .. }
        | TelegramProviderCommand::MarkUnread { account_id, .. }
        | TelegramProviderCommand::Archive { account_id, .. }
        | TelegramProviderCommand::Mute { account_id, .. }
        | TelegramProviderCommand::Join { account_id, .. }
        | TelegramProviderCommand::Leave { account_id, .. }
        | TelegramProviderCommand::AddChatToFolder { account_id, .. }
        | TelegramProviderCommand::RemoveChatFromFolder { account_id, .. }
        | TelegramProviderCommand::ReassignChatFolders { account_id, .. }
        | TelegramProviderCommand::SearchMessages { account_id, .. } => account_id,
        TelegramProviderCommand::ListParticipants { account_id, .. } => account_id,
        TelegramProviderCommand::ListTopics { account_id, .. }
        | TelegramProviderCommand::CreateTopic { account_id, .. }
        | TelegramProviderCommand::SetTopicClosed { account_id, .. } => account_id,
    }
}

pub fn provider_command_operation_id(command: &TelegramProviderCommand) -> &str {
    match command {
        TelegramProviderCommand::SendText(command) => &command.operation_id,
        TelegramProviderCommand::SendMedia(command) => &command.operation_id,
        TelegramProviderCommand::DownloadFile(command) => &command.operation_id,
        TelegramProviderCommand::Reply { operation_id, .. }
        | TelegramProviderCommand::Forward { operation_id, .. }
        | TelegramProviderCommand::Edit { operation_id, .. }
        | TelegramProviderCommand::Delete { operation_id, .. }
        | TelegramProviderCommand::RestoreVisibility { operation_id, .. }
        | TelegramProviderCommand::Reaction { operation_id, .. }
        | TelegramProviderCommand::Pin { operation_id, .. }
        | TelegramProviderCommand::MarkUnread { operation_id, .. }
        | TelegramProviderCommand::Archive { operation_id, .. }
        | TelegramProviderCommand::Mute { operation_id, .. }
        | TelegramProviderCommand::Join { operation_id, .. }
        | TelegramProviderCommand::Leave { operation_id, .. }
        | TelegramProviderCommand::AddChatToFolder { operation_id, .. }
        | TelegramProviderCommand::RemoveChatFromFolder { operation_id, .. }
        | TelegramProviderCommand::ReassignChatFolders { operation_id, .. }
        | TelegramProviderCommand::SearchMessages { operation_id, .. } => operation_id,
        TelegramProviderCommand::ListParticipants { operation_id, .. } => operation_id,
        TelegramProviderCommand::ListTopics { operation_id, .. }
        | TelegramProviderCommand::CreateTopic { operation_id, .. }
        | TelegramProviderCommand::SetTopicClosed { operation_id, .. } => operation_id,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramCommandKind {
    SendText,
    SendMedia,
    DownloadFile,
    Reply,
    Forward,
    Edit,
    Delete,
    RestoreVisibility,
    Reaction,
    Pin,
    MarkUnread,
    Archive,
    Mute,
    Join,
    Leave,
    AddChatToFolder,
    RemoveChatFromFolder,
    ReassignChatFolders,
    SearchMessages,
    ListParticipants,
    ListTopics,
    CreateTopic,
    SetTopicClosed,
}

impl TelegramCommandKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SendText => "send_text",
            Self::SendMedia => "send_media",
            Self::DownloadFile => "download_file",
            Self::Reply => "reply",
            Self::Forward => "forward",
            Self::Edit => "edit",
            Self::Delete => "delete",
            Self::RestoreVisibility => "restore_visibility",
            Self::Reaction => "reaction",
            Self::Pin => "pin",
            Self::MarkUnread => "mark_unread",
            Self::Archive => "archive",
            Self::Mute => "mute",
            Self::Join => "join",
            Self::Leave => "leave",
            Self::AddChatToFolder => "folder_add",
            Self::RemoveChatFromFolder => "folder_remove",
            Self::ReassignChatFolders => "folder_reassign",
            Self::SearchMessages => "search_messages",
            Self::ListParticipants => "list_participants",
            Self::ListTopics => "list_topics",
            Self::CreateTopic => "create_topic",
            Self::SetTopicClosed => "set_topic_closed",
        }
    }
}

pub fn provider_command_kind(command: &TelegramProviderCommand) -> TelegramCommandKind {
    match command {
        TelegramProviderCommand::SendText(_) => TelegramCommandKind::SendText,
        TelegramProviderCommand::SendMedia(_) => TelegramCommandKind::SendMedia,
        TelegramProviderCommand::DownloadFile(_) => TelegramCommandKind::DownloadFile,
        TelegramProviderCommand::Reply { .. } => TelegramCommandKind::Reply,
        TelegramProviderCommand::Forward { .. } => TelegramCommandKind::Forward,
        TelegramProviderCommand::Edit { .. } => TelegramCommandKind::Edit,
        TelegramProviderCommand::Delete { .. } => TelegramCommandKind::Delete,
        TelegramProviderCommand::RestoreVisibility { .. } => TelegramCommandKind::RestoreVisibility,
        TelegramProviderCommand::Reaction { .. } => TelegramCommandKind::Reaction,
        TelegramProviderCommand::Pin { .. } => TelegramCommandKind::Pin,
        TelegramProviderCommand::MarkUnread { .. } => TelegramCommandKind::MarkUnread,
        TelegramProviderCommand::Archive { .. } => TelegramCommandKind::Archive,
        TelegramProviderCommand::Mute { .. } => TelegramCommandKind::Mute,
        TelegramProviderCommand::Join { .. } => TelegramCommandKind::Join,
        TelegramProviderCommand::Leave { .. } => TelegramCommandKind::Leave,
        TelegramProviderCommand::AddChatToFolder { .. } => TelegramCommandKind::AddChatToFolder,
        TelegramProviderCommand::RemoveChatFromFolder { .. } => {
            TelegramCommandKind::RemoveChatFromFolder
        }
        TelegramProviderCommand::ReassignChatFolders { .. } => {
            TelegramCommandKind::ReassignChatFolders
        }
        TelegramProviderCommand::SearchMessages { .. } => TelegramCommandKind::SearchMessages,
        TelegramProviderCommand::ListParticipants { .. } => TelegramCommandKind::ListParticipants,
        TelegramProviderCommand::ListTopics { .. } => TelegramCommandKind::ListTopics,
        TelegramProviderCommand::CreateTopic { .. } => TelegramCommandKind::CreateTopic,
        TelegramProviderCommand::SetTopicClosed { .. } => TelegramCommandKind::SetTopicClosed,
    }
}

pub fn provider_command_chat_id(command: &TelegramProviderCommand) -> Option<&str> {
    match command {
        TelegramProviderCommand::SendText(value) => Some(&value.provider_chat_id),
        TelegramProviderCommand::SendMedia(value) => Some(&value.provider_chat_id),
        TelegramProviderCommand::DownloadFile(_)
        | TelegramProviderCommand::SearchMessages {
            provider_chat_id: None,
            ..
        } => None,
        TelegramProviderCommand::SearchMessages {
            provider_chat_id: Some(provider_chat_id),
            ..
        } => Some(provider_chat_id),
        TelegramProviderCommand::Reply {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::Forward {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::Edit {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::Delete {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::RestoreVisibility {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::Reaction {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::Pin {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::MarkUnread {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::Archive {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::Mute {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::Join {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::Leave {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::AddChatToFolder {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::RemoveChatFromFolder {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::ReassignChatFolders {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::ListParticipants {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::ListTopics {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::CreateTopic {
            provider_chat_id, ..
        }
        | TelegramProviderCommand::SetTopicClosed {
            provider_chat_id, ..
        } => Some(provider_chat_id),
    }
}

pub fn provider_command_message_id(command: &TelegramProviderCommand) -> Option<&str> {
    match command {
        TelegramProviderCommand::Reply {
            reply_to_provider_message_id,
            ..
        } => Some(reply_to_provider_message_id),
        TelegramProviderCommand::Edit {
            provider_message_id,
            ..
        }
        | TelegramProviderCommand::Delete {
            provider_message_id,
            ..
        }
        | TelegramProviderCommand::RestoreVisibility {
            provider_message_id,
            ..
        }
        | TelegramProviderCommand::Reaction {
            provider_message_id,
            ..
        }
        | TelegramProviderCommand::Pin {
            provider_message_id,
            ..
        } => Some(provider_message_id),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramOperationState {
    Accepted,
    Running,
    AwaitingProvider,
    Completed,
    Failed,
    RetryScheduled,
    DeadLetter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TelegramReconciliationState {
    NotObserved,
    AwaitingProvider,
    Observed,
    Mismatch,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramOperation {
    pub operation_id: TelegramOperationId,
    pub account_id: TelegramAccountId,
    pub command_kind: TelegramCommandKind,
    pub idempotency_key: String,
    pub state: TelegramOperationState,
    pub retry_count: u32,
    pub max_retries: u32,
    pub lease_epoch: u64,
    pub reconciliation: TelegramReconciliationState,
    pub last_error: Option<String>,
    pub next_attempt_at_unix_seconds: Option<u64>,
    pub locked_at_unix_seconds: Option<u64>,
    pub locked_by: Option<String>,
    pub provider_observed_at_unix_seconds: Option<u64>,
    pub reconciled_at_unix_seconds: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramContractError {
    EmptyField,
    FieldTooLong,
    InvalidText,
    InvalidPageSize,
    InvalidTransition,
    AccountUnknown,
    AccountRetired,
    RuntimeBlocked,
    RuntimeEpochConflict,
    ReconfigurationCollision,
    ReconfigurationInProgress,
    ReconfigurationUnknown,
    CredentialLeaseRejected,
}

pub fn validate_setup(setup: &TelegramAccountSetup) -> Result<(), TelegramContractError> {
    validate_id(&setup.account_id)?;
    validate_id(&setup.display_name)?;
    if !setup.external_account_id.is_empty() {
        validate_id(&setup.external_account_id)?;
    }
    if setup.qr_authorized {
        return Err(TelegramContractError::InvalidTransition);
    }
    let mut has_api_hash = false;
    let mut has_session_encryption_key = false;
    for binding in &setup.credentials {
        if binding.revision == 0 {
            return Err(TelegramContractError::InvalidTransition);
        }
        let selected = match binding.purpose {
            TelegramCredentialPurpose::ApiHash => &mut has_api_hash,
            TelegramCredentialPurpose::SessionEncryptionKey => &mut has_session_encryption_key,
        };
        if *selected {
            return Err(TelegramContractError::InvalidTransition);
        }
        *selected = true;
    }
    if !has_api_hash || !has_session_encryption_key || setup.credentials.len() != 2 {
        return Err(TelegramContractError::EmptyField);
    }
    Ok(())
}

pub fn validate_runtime_reconfiguration_request(
    request: &TelegramRuntimeReconfigurationRequest,
) -> Result<(), TelegramContractError> {
    match request {
        TelegramRuntimeReconfigurationRequest::Begin {
            reconfiguration_id,
            account_id,
            expected_runtime_epoch,
        } => {
            validate_id(reconfiguration_id)?;
            validate_id(account_id)?;
            if *expected_runtime_epoch == 0 || expected_runtime_epoch.checked_add(1).is_none() {
                return Err(TelegramContractError::RuntimeEpochConflict);
            }
        }
        TelegramRuntimeReconfigurationRequest::Status { reconfiguration_id } => {
            validate_id(reconfiguration_id)?
        }
    }
    Ok(())
}

pub fn validate_text(text: &str) -> Result<(), TelegramContractError> {
    if text.trim().is_empty() {
        return Err(TelegramContractError::EmptyField);
    }
    if text.len() > MAX_TEXT_BYTES {
        return Err(TelegramContractError::InvalidText);
    }
    Ok(())
}

pub fn validate_page_size(size: u32) -> Result<(), TelegramContractError> {
    (size > 0 && size <= MAX_PAGE_SIZE)
        .then_some(())
        .ok_or(TelegramContractError::InvalidPageSize)
}

fn validate_id(value: &str) -> Result<(), TelegramContractError> {
    if value.trim().is_empty() {
        return Err(TelegramContractError::EmptyField);
    }
    if value.len() > MAX_ID_LEN {
        return Err(TelegramContractError::FieldTooLong);
    }
    Ok(())
}

#[cfg(test)]
mod account_setup_tests {
    use super::*;

    fn user_setup() -> TelegramAccountSetup {
        TelegramAccountSetup {
            account_id: "personal-telegram".to_owned(),
            display_name: "Personal Telegram".to_owned(),
            external_account_id: String::new(),
            credentials: vec![
                TelegramCredentialBinding {
                    purpose: TelegramCredentialPurpose::ApiHash,
                    revision: 1,
                },
                TelegramCredentialBinding {
                    purpose: TelegramCredentialPurpose::SessionEncryptionKey,
                    revision: 1,
                },
            ],
            qr_authorized: false,
        }
    }

    #[test]
    fn user_setup_requires_exact_tdlib_credentials() {
        assert_eq!(validate_setup(&user_setup()), Ok(()));

        let mut missing_session_key = user_setup();
        missing_session_key.credentials.pop();
        assert_eq!(
            validate_setup(&missing_session_key),
            Err(TelegramContractError::EmptyField)
        );

        let mut duplicate_api_hash = user_setup();
        duplicate_api_hash.credentials[1].purpose = TelegramCredentialPurpose::ApiHash;
        assert_eq!(
            validate_setup(&duplicate_api_hash),
            Err(TelegramContractError::InvalidTransition)
        );
    }

    #[test]
    fn client_cannot_assert_qr_authorization() {
        let mut setup = user_setup();
        setup.qr_authorized = true;
        assert_eq!(
            validate_setup(&setup),
            Err(TelegramContractError::InvalidTransition)
        );
    }
}
