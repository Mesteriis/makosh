//! TDLib adapter boundary. The provider wire is isolated from Telegram policy and storage.

use std::collections::{HashMap, VecDeque};
use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::{c_char, c_void};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use libloading::Library;
use makosh_telegram_api::{
    TelegramChat, TelegramChatAvatar, TelegramChatFolder, TelegramChatPosition,
    TelegramDownloadFile, TelegramFileSnapshot, TelegramForwardOrigin, TelegramMediaKind,
    TelegramMessageMedia, TelegramMessageObservation, TelegramMessageReferences,
    TelegramParticipant, TelegramParticipantFilter, TelegramParticipantPage,
    TelegramProviderCommand, TelegramProviderEvent, TelegramReplyReference, TelegramSendMessage,
    TelegramTopic, TelegramTypingState, provider_command_operation_id,
    telegram_person_source_identity_v1, validate_provider_command,
};
use makosh_telegram_api::{TelegramChatKind, validate_page_size, validate_text};
use makosh_telegram_call_media_contract::{
    CALL_ENCRYPTION_KEY_BYTES, MAX_READY_TEXT_BYTES, MAX_SERVER_CREDENTIAL_BYTES,
    MAX_SIGNALING_DATA_BYTES, TelegramCallPeerProtocolV1, TelegramCallReadyMaterialV1,
    TelegramCallSecretBytesV1, TelegramCallSecretTextV1, TelegramCallServerKindV1,
    TelegramCallServerV1,
};
use serde_json::{Value, json};
use zeroize::Zeroizing;

pub mod authorization;
pub use authorization::{TdlibAuthorizationDriver, TdlibAuthorizationEvent};

pub const PACKAGE: &str = "makosh-telegram-tdlib";
const MAX_TDLIB_UPDATE_PAYLOADS_PER_POLL: usize = 16;
const MAX_TDLIB_MINITHUMBNAIL_BYTES: usize = 64 * 1024;
// Keep this compatibility inventory aligned with the bundled TDLib schema and
// current td/generate/scheme/td_api.tl. It intentionally includes formats from
// both ends of that supported range. The generic fallback remains
// forward-compatible, while this list prevents a known format from silently
// turning into an empty projection.
#[cfg(test)]
const CURRENT_TDLIB_MESSAGE_CONTENT_TYPES: &[&str] = &[
    "messageAnimatedEmoji",
    "messageAnimation",
    "messageAudio",
    "messageBasicGroupChatCreate",
    "messageBotWriteAccessAllowed",
    "messageCall",
    "messageChatAddedToCommunity",
    "messageChatAddMembers",
    "messageChatBoost",
    "messageChatChangePhoto",
    "messageChatChangeTitle",
    "messageChatDeleteMember",
    "messageChatDeletePhoto",
    "messageChatHasProtectedContentDisableRequested",
    "messageChatHasProtectedContentToggled",
    "messageChatJoinByLink",
    "messageChatJoinByRequest",
    "messageChatOwnerChanged",
    "messageChatOwnerLeft",
    "messageChatRemovedFromCommunity",
    "messageChatSetBackground",
    "messageChatSetMessageAutoDeleteTime",
    "messageChatSetTheme",
    "messageChatSetTtl",
    "messageChatShared",
    "messageChatUpgradeFrom",
    "messageChatUpgradeTo",
    "messageChecklist",
    "messageChecklistTasksAdded",
    "messageChecklistTasksDone",
    "messageContact",
    "messageContactRegistered",
    "messageCustomServiceAction",
    "messageDice",
    "messageDirectMessagePriceChanged",
    "messageDocument",
    "messageExpiredPhoto",
    "messageExpiredVideo",
    "messageExpiredVideoNote",
    "messageExpiredVoiceNote",
    "messageForumTopicCreated",
    "messageForumTopicEdited",
    "messageForumTopicIsClosedToggled",
    "messageForumTopicIsHiddenToggled",
    "messageGame",
    "messageGameScore",
    "messageGift",
    "messageGiftedPremium",
    "messageGiftedStars",
    "messageGiftedTon",
    "messageGiveaway",
    "messageGiveawayCompleted",
    "messageGiveawayCreated",
    "messageGiveawayPrizeStars",
    "messageGiveawayWinners",
    "messageGroupCall",
    "messageInviteVideoChatParticipants",
    "messageInvoice",
    "messageLiveLocation",
    "messageLocation",
    "messageManagedBotCreated",
    "messagePaidMedia",
    "messagePaidMessagePriceChanged",
    "messagePaidMessagesRefunded",
    "messagePassportDataReceived",
    "messagePassportDataSent",
    "messagePaymentRefunded",
    "messagePaymentSuccessful",
    "messagePaymentSuccessfulBot",
    "messagePhoto",
    "messagePinMessage",
    "messagePoll",
    "messagePollOptionAdded",
    "messagePollOptionDeleted",
    "messagePremiumGiftCode",
    "messageProximityAlertTriggered",
    "messageRefundedUpgradedGift",
    "messageRichMessage",
    "messageScreenshotTaken",
    "messageStakeDice",
    "messageSticker",
    "messageStory",
    "messageSuggestBirthdate",
    "messageSuggestedPostApprovalFailed",
    "messageSuggestedPostApproved",
    "messageSuggestedPostDeclined",
    "messageSuggestedPostPaid",
    "messageSuggestedPostRefunded",
    "messageSuggestProfilePhoto",
    "messageSupergroupChatCreate",
    "messageText",
    "messageUpgradedGift",
    "messageUpgradedGiftPurchaseOffer",
    "messageUpgradedGiftPurchaseOfferRejected",
    "messageUnsupported",
    "messageUsersShared",
    "messageVenue",
    "messageVideo",
    "messageVideoChatEnded",
    "messageVideoChatScheduled",
    "messageVideoChatStarted",
    "messageVideoNote",
    "messageVoiceNote",
    "messageWebAppDataReceived",
    "messageWebAppDataSent",
    "messageWebsiteConnected",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TdlibCallDirection {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TdlibCallState {
    Pending,
    ExchangingKeys,
    Ready,
    HangingUp,
    Discarded,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TdlibCallDiscardReason {
    Empty,
    Missed,
    Declined,
    Disconnected,
    HungUp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TdlibCallFailureCategory {
    Network,
    NotAvailable,
    Permission,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TdlibCallObservation {
    pub account_id: String,
    pub tdlib_call_id: i32,
    pub provider_call_unique_id: Option<i64>,
    pub provider_user_id: String,
    pub direction: TdlibCallDirection,
    pub is_video: bool,
    pub state: TdlibCallState,
    pub pending_created: bool,
    pub pending_received: bool,
    pub discard_reason: Option<TdlibCallDiscardReason>,
    pub failure_category: Option<TdlibCallFailureCategory>,
}

#[derive(Debug)]
pub enum TdlibProviderUpdate {
    Operational(Box<TelegramProviderEvent>),
    DownloadedFile(TdlibDownloadedFile),
    Call(TdlibCallObservation),
    CallReady {
        observation: TdlibCallObservation,
        material: TelegramCallReadyMaterialV1,
    },
    CallSignaling {
        account_id: String,
        tdlib_call_id: i32,
        data: TelegramCallSecretBytesV1,
    },
}

#[derive(Clone, Eq, PartialEq)]
pub struct TdlibDownloadedFile {
    pub snapshot: TelegramFileSnapshot,
    pub local_path: PathBuf,
}

impl fmt::Debug for TdlibDownloadedFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TdlibDownloadedFile")
            .field("snapshot", &self.snapshot)
            .field("local_path", &"[redacted]")
            .finish()
    }
}

/// Runtime-owned port for converting an authorized opaque BlobRef into a
/// short-lived TDLib input file. The adapter never reads a filesystem path
/// from the provider contract.
pub trait TelegramMediaMaterializer {
    fn materialize(&mut self, blob_ref: &str) -> Result<String, TdlibError>;
    fn release(&mut self, materialized_path: &str);
}

#[derive(Clone, Eq, PartialEq)]
pub struct TdlibAuthorizationParameters {
    pub api_id: i64,
    pub api_hash: Zeroizing<String>,
    pub database_directory: PathBuf,
    pub session_encryption_key: Option<Zeroizing<Vec<u8>>>,
}

impl fmt::Debug for TdlibAuthorizationParameters {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TdlibAuthorizationParameters")
            .field("api_id", &"[redacted]")
            .field("api_hash", &"[redacted]")
            .field("database_directory", &"[redacted]")
            .field(
                "session_encryption_key",
                &self.session_encryption_key.as_ref().map(|_| "[redacted]"),
            )
            .finish()
    }
}

impl TdlibAuthorizationParameters {
    pub fn from_secret_material(
        api_id: i64,
        api_hash: Zeroizing<Vec<u8>>,
        database_directory: PathBuf,
        session_encryption_key: Option<Zeroizing<Vec<u8>>>,
    ) -> Result<Self, TdlibError> {
        let api_hash = String::from_utf8(api_hash.to_vec())
            .map(Zeroizing::new)
            .map_err(|_| TdlibError::Protocol("Telegram API hash is not UTF-8".to_owned()))?;
        if api_id <= 0 || api_hash.trim().is_empty() {
            return Err(TdlibError::Protocol(
                "Telegram application credentials are invalid".to_owned(),
            ));
        }
        Ok(Self {
            api_id,
            api_hash,
            database_directory,
            session_encryption_key,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum TdlibAuthorizationUpdate {
    WaitingParameters,
    WaitingEncryptionKey,
    WaitingQrScan,
    WaitingPassword { hint: Option<String> },
    Ready,
    Closing,
    Closed,
    Error { code: Option<i64>, message: String },
    Other(String),
}

impl fmt::Debug for TdlibAuthorizationUpdate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WaitingParameters => formatter.write_str("WaitingParameters"),
            Self::WaitingEncryptionKey => formatter.write_str("WaitingEncryptionKey"),
            Self::WaitingQrScan => formatter.write_str("WaitingQrScan"),
            Self::WaitingPassword { hint } => formatter
                .debug_struct("WaitingPassword")
                .field("hint", &hint.as_ref().map(|_| "[redacted]"))
                .finish(),
            Self::Ready => formatter.write_str("Ready"),
            Self::Closing => formatter.write_str("Closing"),
            Self::Closed => formatter.write_str("Closed"),
            Self::Error { code, .. } => formatter
                .debug_struct("Error")
                .field("code", code)
                .field("message", &"[redacted]")
                .finish(),
            Self::Other(_) => formatter.write_str("Other([redacted])"),
        }
    }
}

pub fn set_tdlib_parameters_request(
    parameters: &TdlibAuthorizationParameters,
) -> Result<Value, TdlibError> {
    if parameters.api_id <= 0 || parameters.api_hash.trim().is_empty() {
        return Err(TdlibError::Protocol(
            "TDLib application credentials are invalid".to_owned(),
        ));
    }
    let database_directory = parameters.database_directory.to_string_lossy().into_owned();
    let files_directory = parameters
        .database_directory
        .join("files")
        .to_string_lossy()
        .into_owned();
    let encryption_key = parameters
        .session_encryption_key
        .as_deref()
        .map(|value| STANDARD.encode(value))
        .unwrap_or_default();
    Ok(json!({
        "@type": "setTdlibParameters",
        "parameters": {
            "use_test_dc": false,
            "database_directory": database_directory,
            "files_directory": files_directory,
            "database_encryption_key": encryption_key,
            "use_file_database": true,
            "use_chat_info_database": true,
            "use_message_database": true,
            "use_secret_chats": false,
            "api_id": parameters.api_id,
            "api_hash": parameters.api_hash.as_str(),
            "system_language_code": "en",
            "device_model": "Макошь",
            "system_version": std::env::consts::OS,
            "application_version": env!("CARGO_PKG_VERSION"),
            "enable_storage_optimizer": true,
            "ignore_file_names": false
        },
        "@extra": "makosh-set-tdlib-parameters"
    }))
}

pub fn check_database_encryption_key_request(key: Option<&[u8]>) -> Value {
    json!({
        "@type": "checkDatabaseEncryptionKey",
        "encryption_key": key.map(|value| STANDARD.encode(value)).unwrap_or_default(),
        "@extra": "makosh-check-database-encryption-key"
    })
}

pub fn request_qr_code_authentication() -> Value {
    json!({
        "@type": "requestQrCodeAuthentication",
        "other_user_ids": [],
        "@extra": "makosh-request-qr-code-authentication"
    })
}

pub fn check_authentication_password(password: &str) -> Result<Value, TdlibError> {
    if password.is_empty() {
        return Err(TdlibError::Protocol(
            "Telegram password is empty".to_owned(),
        ));
    }
    Ok(json!({
        "@type": "checkAuthenticationPassword",
        "password": password,
        "@extra": "makosh-check-authentication-password"
    }))
}

pub fn close_session_request() -> Value {
    json!({"@type": "close", "@extra": "makosh-close-tdlib-session"})
}

fn disable_tdlib_logging_request() -> Value {
    json!({
        "@type": "setLogVerbosityLevel",
        "new_verbosity_level": 0,
    })
}

pub fn parse_authorization_update(payload: &Value) -> Result<TdlibAuthorizationUpdate, TdlibError> {
    let authorization_state = authorization_state(payload);
    let state = authorization_state
        .get("@type")
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib authorization state is missing".to_owned()))?;
    Ok(match state {
        "authorizationStateWaitTdlibParameters" => TdlibAuthorizationUpdate::WaitingParameters,
        "authorizationStateWaitEncryptionKey" => TdlibAuthorizationUpdate::WaitingEncryptionKey,
        "authorizationStateWaitOtherDeviceConfirmation" => TdlibAuthorizationUpdate::WaitingQrScan,
        "authorizationStateWaitPassword" => TdlibAuthorizationUpdate::WaitingPassword {
            hint: authorization_state
                .get("password_hint")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        },
        "authorizationStateReady" => TdlibAuthorizationUpdate::Ready,
        "authorizationStateClosing" | "authorizationStateLoggingOut" => {
            TdlibAuthorizationUpdate::Closing
        }
        "authorizationStateClosed" => TdlibAuthorizationUpdate::Closed,
        "error" => TdlibAuthorizationUpdate::Error {
            code: authorization_state.get("code").and_then(Value::as_i64),
            message: "TDLib authorization error".to_owned(),
        },
        other => TdlibAuthorizationUpdate::Other(other.to_owned()),
    })
}

pub(crate) fn parse_qr_authorization_link(payload: &Value) -> Result<String, TdlibError> {
    authorization_state(payload)
        .get("link")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TdlibError::Protocol("TDLib QR authorization state did not include a link".to_owned())
        })
}

fn authorization_state(payload: &Value) -> &Value {
    payload.get("authorization_state").unwrap_or(payload)
}

#[derive(Debug)]
pub enum TdlibRequest {
    GetOwnUser {
        correlation_id: String,
    },
    ResolveSender {
        correlation_id: String,
        provider_sender_id: String,
    },
    CreateCall {
        operation_id: String,
        provider_user_id: String,
        protocol: makosh_telegram_call_media_contract::TelegramCallProtocolV1,
    },
    AcceptCall {
        operation_id: String,
        tdlib_call_id: i32,
        protocol: makosh_telegram_call_media_contract::TelegramCallProtocolV1,
    },
    DiscardCall {
        operation_id: String,
        tdlib_call_id: i32,
        is_disconnected: bool,
        duration_seconds: u32,
        connection_id: i64,
    },
    SendCallSignalingData {
        correlation_id: String,
        tdlib_call_id: i32,
        data: TelegramCallSecretBytesV1,
    },
    LoadChats {
        account_id: String,
        limit: u32,
    },
    LoadHistory {
        account_id: String,
        provider_chat_id: String,
        from_message_id: Option<i64>,
        mode: makosh_telegram_api::TelegramHistorySyncMode,
        limit: u32,
    },
    GetMessage {
        account_id: String,
        provider_chat_id: String,
        provider_message_id: String,
    },
    SendMessage(TelegramSendMessage),
    SendMedia(makosh_telegram_api::TelegramSendMedia),
    SendMediaMaterialized {
        command: makosh_telegram_api::TelegramSendMedia,
        materialized_path: String,
    },
    DownloadFile(TelegramDownloadFile),
    ListParticipants {
        account_id: String,
        provider_chat_id: String,
        filter: TelegramParticipantFilter,
        offset: u32,
        limit: u32,
    },
    ListBasicGroupParticipants {
        account_id: String,
        provider_chat_id: String,
        basic_group_id: i64,
    },
    ListTopics {
        account_id: String,
        provider_chat_id: String,
        limit: u32,
    },
    GetChatFolder {
        account_id: String,
        provider_folder_id: i64,
    },
    ProviderCommand(TelegramProviderCommand),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TdlibResponse {
    OwnUser {
        provider_user_id: String,
    },
    SenderName {
        provider_sender_id: String,
        display_name: Option<String>,
    },
    CallCreated {
        operation_id: String,
        tdlib_call_id: i32,
    },
    Chats(Vec<TelegramChat>),
    History(Vec<TelegramMessageObservation>),
    Message(TelegramMessageObservation),
    Sent {
        provider_message_id: String,
    },
    File(TelegramFileSnapshot),
    Participants(TelegramParticipantPage),
    Topics(Vec<TelegramTopic>),
    ChatFolders(Vec<TelegramChatFolder>),
    FolderReassigned {
        added_provider_folder_ids: Vec<i64>,
        removed_provider_folder_ids: Vec<i64>,
    },
    Accepted {
        operation_id: String,
    },
}

pub fn get_chats_request(account_id: &str, limit: u32) -> Result<TdlibRequest, TdlibError> {
    validate_page_size(limit)
        .map_err(|_| TdlibError::Protocol("invalid chat page size".to_owned()))?;
    if account_id.trim().is_empty() {
        return Err(TdlibError::Protocol("account id is empty".to_owned()));
    }
    Ok(TdlibRequest::LoadChats {
        account_id: account_id.to_owned(),
        limit,
    })
}

pub fn get_history_request(
    account_id: &str,
    provider_chat_id: &str,
    limit: u32,
) -> Result<TdlibRequest, TdlibError> {
    get_history_request_with_options(
        account_id,
        provider_chat_id,
        None,
        makosh_telegram_api::TelegramHistorySyncMode::Latest,
        limit,
    )
}

pub fn get_history_request_with_options(
    account_id: &str,
    provider_chat_id: &str,
    from_message_id: Option<i64>,
    mode: makosh_telegram_api::TelegramHistorySyncMode,
    limit: u32,
) -> Result<TdlibRequest, TdlibError> {
    validate_page_size(limit)
        .map_err(|_| TdlibError::Protocol("invalid history page size".to_owned()))?;
    provider_chat_id
        .parse::<i64>()
        .map_err(|_| TdlibError::Protocol("provider chat id is not an integer".to_owned()))?;
    if account_id.trim().is_empty() {
        return Err(TdlibError::Protocol("account id is empty".to_owned()));
    }
    if from_message_id.is_some_and(|message_id| message_id <= 0)
        || (matches!(mode, makosh_telegram_api::TelegramHistorySyncMode::Older)
            && from_message_id.is_none())
    {
        return Err(TdlibError::Protocol("history cursor is invalid".to_owned()));
    }
    Ok(TdlibRequest::LoadHistory {
        account_id: account_id.to_owned(),
        provider_chat_id: provider_chat_id.to_owned(),
        from_message_id,
        mode,
        limit,
    })
}

pub fn get_message_request(
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
) -> Result<TdlibRequest, TdlibError> {
    if account_id.trim().is_empty() {
        return Err(TdlibError::Protocol("account id is empty".to_owned()));
    }
    signed_chat_id(provider_chat_id)?;
    provider_id(provider_message_id)?;
    Ok(TdlibRequest::GetMessage {
        account_id: account_id.to_owned(),
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
    })
}

pub fn send_message_request(command: TelegramSendMessage) -> Result<TdlibRequest, TdlibError> {
    validate_text(&command.text)
        .map_err(|_| TdlibError::Protocol("message text is invalid".to_owned()))?;
    command
        .provider_chat_id
        .parse::<i64>()
        .map_err(|_| TdlibError::Protocol("provider chat id is not an integer".to_owned()))?;
    Ok(TdlibRequest::SendMessage(command))
}

pub fn encode_request(request: &TdlibRequest) -> Result<Value, TdlibError> {
    match request {
        TdlibRequest::GetOwnUser { correlation_id } => Ok(json!({
            "@type": "getMe",
            "@extra": correlation_id,
        })),
        TdlibRequest::ResolveSender {
            correlation_id,
            provider_sender_id,
        } => {
            let provider_sender_id = signed_chat_id(provider_sender_id)?;
            Ok(if provider_sender_id > 0 {
                json!({
                    "@type": "getUser",
                    "user_id": provider_sender_id,
                    "@extra": correlation_id,
                })
            } else {
                json!({
                    "@type": "getChat",
                    "chat_id": provider_sender_id,
                    "@extra": correlation_id,
                })
            })
        }
        TdlibRequest::CreateCall {
            operation_id,
            provider_user_id,
            protocol,
        } => Ok(json!({
            "@type": "createCall",
            "user_id": provider_id(provider_user_id)?,
            "protocol": call_protocol_value(protocol)?,
            "is_video": false,
            "@extra": operation_id,
        })),
        TdlibRequest::AcceptCall {
            operation_id,
            tdlib_call_id,
            protocol,
        } => {
            if *tdlib_call_id <= 0 {
                return Err(TdlibError::Protocol(
                    "TDLib call identifier is invalid".to_owned(),
                ));
            }
            Ok(json!({
                "@type": "acceptCall",
                "call_id": tdlib_call_id,
                "protocol": call_protocol_value(protocol)?,
                "@extra": operation_id,
            }))
        }
        TdlibRequest::DiscardCall {
            operation_id,
            tdlib_call_id,
            is_disconnected,
            duration_seconds,
            connection_id,
        } => {
            if *tdlib_call_id <= 0 || i32::try_from(*duration_seconds).is_err() {
                return Err(TdlibError::Protocol(
                    "TDLib call discard context is invalid".to_owned(),
                ));
            }
            Ok(json!({
                "@type": "discardCall",
                "call_id": tdlib_call_id,
                "is_disconnected": is_disconnected,
                "invite_link": "",
                "duration": duration_seconds,
                "is_video": false,
                "connection_id": connection_id,
                "@extra": operation_id,
            }))
        }
        TdlibRequest::SendCallSignalingData {
            correlation_id,
            tdlib_call_id,
            data,
        } => {
            if *tdlib_call_id <= 0 {
                return Err(TdlibError::Protocol(
                    "TDLib call identifier is invalid".to_owned(),
                ));
            }
            Ok(json!({
                "@type": "sendCallSignalingData",
                "call_id": tdlib_call_id,
                "data": STANDARD.encode(data.expose()),
                "@extra": correlation_id,
            }))
        }
        TdlibRequest::LoadChats { account_id, limit } => Ok(json!({
            "@type": "getChats",
            "chat_list": null,
            "limit": limit,
            "@extra": account_id,
        })),
        TdlibRequest::LoadHistory {
            account_id,
            provider_chat_id,
            from_message_id,
            limit,
            ..
        } => Ok(json!({
            "@type": "getChatHistory",
            "chat_id": signed_chat_id(provider_chat_id)?,
            "from_message_id": from_message_id.unwrap_or_default(),
            "offset": 0,
            "limit": limit,
            "only_local": false,
            "@extra": account_id,
        })),
        TdlibRequest::GetMessage {
            account_id,
            provider_chat_id,
            provider_message_id,
        } => Ok(json!({
            "@type": "getMessage",
            "chat_id": signed_chat_id(provider_chat_id)?,
            "message_id": provider_id(provider_message_id)?,
            "@extra": format!("{account_id}:message:{provider_message_id}"),
        })),
        TdlibRequest::SendMessage(command) => Ok(json!({
            "@type": "sendMessage",
            "chat_id": signed_chat_id(&command.provider_chat_id)?,
            "reply_to": {"@type": "inputMessageReplyToMessage", "message_id": 0},
            "options": null,
            "reply_markup": null,
            "input_message_content": {
                "@type": "inputMessageText",
                "text": {"@type": "formattedText", "text": command.text},
                "clear_draft": false,
                "link_preview_options": null,
            },
            "@extra": command.operation_id,
        })),
        TdlibRequest::SendMedia(_) => Err(TdlibError::Protocol(
            "Telegram media request requires an authorized Blob materializer".to_owned(),
        )),
        TdlibRequest::SendMediaMaterialized {
            command,
            materialized_path,
        } => encode_send_media_materialized(command, materialized_path),
        TdlibRequest::DownloadFile(command) => Ok(json!({
            "@type": "downloadFile",
            "file_id": provider_id(&command.provider_file_id)?,
            "priority": command.priority,
            "offset": 0,
            "limit": 0,
            "synchronous": false,
            "@extra": command.operation_id,
        })),
        TdlibRequest::ListParticipants {
            account_id,
            provider_chat_id,
            filter,
            offset,
            limit,
        } => Ok(json!({
            "@type": "getSupergroupMembers",
            "supergroup_id": provider_id(provider_chat_id)?,
            "filter": {"@type": match filter {
                TelegramParticipantFilter::Recent => "supergroupMembersFilterRecent",
                TelegramParticipantFilter::Administrators => "supergroupMembersFilterAdministrators",
            }},
            "offset": offset,
            "limit": limit,
            "@extra": account_id,
        })),
        TdlibRequest::ListBasicGroupParticipants {
            account_id,
            basic_group_id,
            ..
        } => Ok(json!({
            "@type": "getBasicGroup",
            "basic_group_id": basic_group_id,
            "@extra": account_id,
        })),
        TdlibRequest::ListTopics {
            account_id,
            provider_chat_id,
            limit,
        } => Ok(json!({
            "@type": "getForumTopics",
            "chat_id": signed_chat_id(provider_chat_id)?,
            "query": "",
            "offset_date": 0,
            "offset_message_id": 0,
            "limit": limit,
            "@extra": account_id,
        })),
        TdlibRequest::GetChatFolder {
            account_id,
            provider_folder_id,
        } => Ok(json!({
            "@type": "getChatFolder",
            "chat_folder_id": provider_folder_id,
            "@extra": format!("{account_id}:folder:{provider_folder_id}"),
        })),
        TdlibRequest::ProviderCommand(command) => encode_provider_command(command),
    }
}

fn call_protocol_value(
    protocol: &makosh_telegram_call_media_contract::TelegramCallProtocolV1,
) -> Result<Value, TdlibError> {
    protocol
        .validate()
        .map_err(|_| TdlibError::Protocol("Telegram call protocol is invalid".to_owned()))?;
    Ok(json!({
        "@type": "callProtocol",
        "udp_p2p": protocol.udp_p2p,
        "udp_reflector": protocol.udp_reflector,
        "min_layer": protocol.min_layer,
        "max_layer": protocol.max_layer,
        "library_versions": protocol.library_versions,
    }))
}

pub fn encode_provider_command(command: &TelegramProviderCommand) -> Result<Value, TdlibError> {
    validate_provider_command(command)
        .map_err(|_| TdlibError::Protocol("Telegram provider command is invalid".to_owned()))?;
    match command {
        TelegramProviderCommand::SendText(command) => {
            encode_request(&TdlibRequest::SendMessage(command.clone()))
        }
        TelegramProviderCommand::SendMedia(_) => Err(TdlibError::Protocol(
            "Telegram media command requires an authorized Blob materializer".to_owned(),
        )),
        TelegramProviderCommand::DownloadFile(command) => {
            encode_request(&TdlibRequest::DownloadFile(command.clone()))
        }
        TelegramProviderCommand::ListTopics {
            operation_id,
            account_id,
            provider_chat_id,
            limit,
        } => {
            let request = TdlibRequest::ListTopics {
                account_id: account_id.clone(),
                provider_chat_id: provider_chat_id.clone(),
                limit: *limit,
            };
            let mut encoded = encode_request(&request)?;
            encoded["@extra"] = json!(operation_id);
            Ok(encoded)
        }
        TelegramProviderCommand::CreateTopic {
            operation_id,
            provider_chat_id,
            title,
            ..
        } => Ok(json!({
            "@type": "createForumTopic",
            "chat_id": signed_chat_id(provider_chat_id)?,
            "name": title,
            "icon": {"@type": "messageForumTopicIcon", "color": 7322096, "custom_emoji_id": ""},
            "@extra": operation_id,
        })),
        TelegramProviderCommand::SetTopicClosed {
            operation_id,
            provider_chat_id,
            provider_topic_id,
            is_closed,
            ..
        } => Ok(json!({
            "@type": "toggleForumTopicIsClosed",
            "chat_id": signed_chat_id(provider_chat_id)?,
            "message_thread_id": provider_id(provider_topic_id)?,
            "is_closed": is_closed,
            "@extra": operation_id,
        })),
        TelegramProviderCommand::Reply {
            operation_id,
            provider_chat_id,
            reply_to_provider_message_id,
            text,
            ..
        } => Ok(text_command(
            "sendMessage",
            provider_chat_id,
            Some(reply_to_provider_message_id),
            text,
            operation_id,
        )?),
        TelegramProviderCommand::Forward {
            operation_id,
            provider_chat_id,
            from_provider_chat_id,
            from_provider_message_id,
            ..
        } => Ok(json!({
            "@type": "forwardMessages",
            "chat_id": signed_chat_id(provider_chat_id)?,
            "from_chat_id": signed_chat_id(from_provider_chat_id)?,
            "message_ids": [provider_id(from_provider_message_id)?],
            "options": null,
            "send_copy": false,
            "remove_caption": false,
            "@extra": operation_id,
        })),
        TelegramProviderCommand::Edit {
            operation_id,
            provider_chat_id,
            provider_message_id,
            text,
            ..
        } => Ok(json!({
            "@type": "editMessageText",
            "chat_id": signed_chat_id(provider_chat_id)?,
            "message_id": provider_id(provider_message_id)?,
            "input_message_content": formatted_text_content(text, false),
            "@extra": operation_id,
        })),
        TelegramProviderCommand::Delete {
            operation_id,
            provider_chat_id,
            provider_message_id,
            revoke,
            ..
        } => Ok(json!({
            "@type": "deleteMessages",
            "chat_id": signed_chat_id(provider_chat_id)?,
            "message_ids": [provider_id(provider_message_id)?],
            "revoke": revoke,
            "@extra": operation_id,
        })),
        TelegramProviderCommand::RestoreVisibility { .. } => Err(TdlibError::Protocol(
            "Telegram restore visibility is local-only".to_owned(),
        )),
        TelegramProviderCommand::Reaction {
            operation_id,
            provider_chat_id,
            provider_message_id,
            emoji,
            active,
            ..
        } => Ok(json!({
            "@type": if *active { "addMessageReaction" } else { "removeMessageReaction" },
            "chat_id": signed_chat_id(provider_chat_id)?,
            "message_id": provider_id(provider_message_id)?,
            "reaction_type": {"@type": "reactionTypeEmoji", "emoji": emoji},
            "is_big": false,
            "update_recent_reactions": true,
            "@extra": operation_id,
        })),
        TelegramProviderCommand::Pin {
            operation_id,
            provider_chat_id,
            provider_message_id,
            active,
            ..
        } => Ok(if *active {
            json!({
                "@type": "pinChatMessage", "chat_id": signed_chat_id(provider_chat_id)?,
                "message_id": provider_id(provider_message_id)?, "disable_notification": false,
                "only_for_self": false, "@extra": operation_id
            })
        } else {
            json!({
                "@type": "unpinChatMessage", "chat_id": signed_chat_id(provider_chat_id)?,
                "message_id": provider_id(provider_message_id)?, "@extra": operation_id
            })
        }),
        TelegramProviderCommand::MarkUnread {
            operation_id,
            provider_chat_id,
            unread,
            read_through_provider_message_id,
            ..
        } => {
            if !unread && let Some(message_id) = read_through_provider_message_id {
                return Ok(json!({
                    "@type": "viewMessages", "chat_id": signed_chat_id(provider_chat_id)?,
                    "message_ids": [provider_id(message_id)?], "source": null,
                    "force_read": true, "@extra": operation_id
                }));
            }
            Ok(json!({
                "@type": "toggleChatIsMarkedAsUnread", "chat_id": signed_chat_id(provider_chat_id)?,
                "is_marked_as_unread": unread, "@extra": operation_id
            }))
        }
        TelegramProviderCommand::Archive {
            operation_id,
            provider_chat_id,
            archived,
            ..
        } => Ok(json!({
            "@type": "addChatToList", "chat_id": signed_chat_id(provider_chat_id)?,
            "chat_list": {"@type": if *archived { "chatListArchive" } else { "chatListMain" }},
            "@extra": operation_id
        })),
        TelegramProviderCommand::Mute {
            operation_id,
            provider_chat_id,
            muted,
            ..
        } => Ok(json!({
            "@type": "setChatNotificationSettings", "chat_id": signed_chat_id(provider_chat_id)?,
            "notification_settings": {"@type": "chatNotificationSettings", "use_default_mute_for": !muted,
                "mute_for": if *muted { 31_708_800 } else { 0 }, "use_default_sound": true,
                "sound_id": 0, "use_default_show_preview": true, "show_preview": true,
                "use_default_mute_stories": true, "mute_stories": false,
                "use_default_story_sound": true, "story_sound_id": 0,
                "use_default_show_story_poster": true, "show_story_poster": true,
                "use_default_disable_pinned_message_notifications": true,
                "disable_pinned_message_notifications": false,
                "use_default_disable_mention_notifications": true,
                "disable_mention_notifications": false},
            "@extra": operation_id
        })),
        TelegramProviderCommand::Join {
            operation_id,
            provider_chat_id,
            ..
        } => Ok(json!({
            "@type": "joinChat", "chat_id": signed_chat_id(provider_chat_id)?, "@extra": operation_id
        })),
        TelegramProviderCommand::Leave {
            operation_id,
            provider_chat_id,
            ..
        } => Ok(json!({
            "@type": "leaveChat", "chat_id": signed_chat_id(provider_chat_id)?, "@extra": operation_id
        })),
        TelegramProviderCommand::AddChatToFolder {
            operation_id,
            provider_chat_id,
            provider_folder_id,
            ..
        } => Ok(json!({
            "@type": "addChatToList",
            "chat_id": signed_chat_id(provider_chat_id)?,
            "chat_list": {"@type": "chatListFolder", "chat_folder_id": provider_folder_id},
            "@extra": operation_id
        })),
        TelegramProviderCommand::RemoveChatFromFolder {
            operation_id,
            provider_folder_id,
            ..
        } => Ok(json!({
            "@type": "getChatFolder",
            "chat_folder_id": provider_folder_id,
            "@extra": format!("{operation_id}:get")
        })),
        TelegramProviderCommand::ReassignChatFolders {
            operation_id,
            provider_chat_id,
            ..
        } => Ok(json!({
            "@type": "getChat",
            "chat_id": signed_chat_id(provider_chat_id)?,
            "@extra": format!("{operation_id}:get-chat")
        })),
        TelegramProviderCommand::SearchMessages {
            operation_id,
            provider_chat_id,
            query,
            limit,
            ..
        } => {
            let query = query.trim();
            if let Some(chat_id) = provider_chat_id {
                Ok(json!({
                    "@type": "searchChatMessages", "chat_id": signed_chat_id(chat_id)?,
                    "query": query, "sender_id": null, "from_message_id": 0, "offset": 0,
                    "limit": limit, "filter": {"@type": "searchMessagesFilterEmpty"},
                    "@extra": operation_id
                }))
            } else {
                Ok(json!({
                    "@type": "searchMessages", "chat_list": {"@type": "chatListMain"},
                    "query": query, "offset_date": 0, "offset_chat_id": 0,
                    "offset_message_id": 0, "limit": limit,
                    "filter": {"@type": "searchMessagesFilterEmpty"}, "@extra": operation_id
                }))
            }
        }
        TelegramProviderCommand::ListParticipants {
            operation_id,
            account_id,
            provider_chat_id,
            filter,
            offset,
            limit,
        } => {
            let request = TdlibRequest::ListParticipants {
                account_id: account_id.clone(),
                provider_chat_id: provider_chat_id.clone(),
                filter: *filter,
                offset: *offset,
                limit: *limit,
            };
            let mut encoded = encode_request(&request)?;
            encoded["@extra"] = json!(operation_id);
            Ok(encoded)
        }
    }
}

pub fn encode_send_media_materialized(
    command: &makosh_telegram_api::TelegramSendMedia,
    materialized_path: &str,
) -> Result<Value, TdlibError> {
    let input_type = match command.media_kind {
        TelegramMediaKind::Photo => "inputMessagePhoto",
        TelegramMediaKind::Video => "inputMessageVideo",
        TelegramMediaKind::Audio => "inputMessageAudio",
        TelegramMediaKind::Document => "inputMessageDocument",
        TelegramMediaKind::Animation => "inputMessageAnimation",
        TelegramMediaKind::VoiceNote => "inputMessageVoiceNote",
    };
    let file_field = match command.media_kind {
        TelegramMediaKind::Photo | TelegramMediaKind::Video | TelegramMediaKind::Animation => {
            "photo"
        }
        TelegramMediaKind::Audio => "audio",
        TelegramMediaKind::Document => "document",
        TelegramMediaKind::VoiceNote => "voice_note",
    };
    let caption = command.caption.as_deref().unwrap_or("");
    let mut content = json!({
        "@type": input_type,
        "caption": {"@type": "formattedText", "text": caption, "entities": []},
    });
    if materialized_path.trim().is_empty() {
        return Err(TdlibError::Protocol(
            "Telegram media materialization path is empty".to_owned(),
        ));
    }
    content[file_field] = json!({"@type": "inputFileLocal", "path": materialized_path});
    Ok(json!({
        "@type": "sendMessage",
        "chat_id": signed_chat_id(&command.provider_chat_id)?,
        "reply_to": {"@type": "inputMessageReplyToMessage", "message_id": 0},
        "options": null,
        "reply_markup": null,
        "input_message_content": content,
        "@extra": command.operation_id,
    }))
}

fn text_command(
    command_type: &str,
    provider_chat_id: &str,
    reply_to_provider_message_id: Option<&str>,
    text: &str,
    operation_id: &str,
) -> Result<Value, TdlibError> {
    let mut request = json!({
        "@type": command_type,
        "chat_id": signed_chat_id(provider_chat_id)?,
        "input_message_content": formatted_text_content(text, true),
        "@extra": operation_id,
    });
    if let Some(message_id) = reply_to_provider_message_id {
        request["reply_to"] = json!({
            "@type": "inputMessageReplyToMessage",
            "message_id": provider_id(message_id)?,
        });
    }
    Ok(request)
}

fn formatted_text_content(text: &str, clear_draft: bool) -> Value {
    json!({
        "@type": "inputMessageText",
        "text": {"@type": "formattedText", "text": text, "entities": []},
        "clear_draft": clear_draft,
        "link_preview_options": null,
    })
}

fn provider_id(value: &str) -> Result<i64, TdlibError> {
    value
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| TdlibError::Protocol("Telegram provider id is invalid".to_owned()))
}

fn signed_chat_id(value: &str) -> Result<i64, TdlibError> {
    value
        .parse::<i64>()
        .ok()
        .filter(|value| *value != 0)
        .ok_or_else(|| TdlibError::Protocol("Telegram provider chat id is invalid".to_owned()))
}

pub fn parse_chat(account_id: &str, payload: &Value) -> Result<TelegramChat, TdlibError> {
    let provider_chat_id = required_string(payload, "id")?;
    let title = required_string(payload, "title")?;
    let kind = payload
        .get("type")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .map(chat_kind)
        .ok_or_else(|| TdlibError::Protocol("TDLib chat type is missing".to_owned()))??;
    let username = payload
        .get("usernames")
        .and_then(|value| value.get("editable_username"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let avatar = payload
        .get("photo")
        .filter(|value| !value.is_null())
        .and_then(|value| value.get("small").or_else(|| value.get("big")));
    Ok(TelegramChat {
        account_id: account_id.to_owned(),
        provider_chat_id,
        kind,
        title,
        username,
        avatar_provider_file_id: avatar
            .and_then(|value| value.get("id"))
            .and_then(value_id_optional),
        avatar_provider_unique_id: avatar
            .and_then(|value| value.get("remote"))
            .and_then(|value| value.get("unique_id"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

fn parse_new_chat_update(
    account_id: &str,
    payload: &Value,
) -> Result<Option<TelegramChat>, TdlibError> {
    let chat_payload = match payload.get("@type").and_then(Value::as_str) {
        Some("updateNewChat") => payload
            .get("chat")
            .ok_or_else(|| TdlibError::Protocol("updateNewChat has no chat".to_owned()))?,
        // getChat returns a plain `chat` object. Sender resolution for
        // channel posts and anonymous administrators uses messageSenderChat,
        // so the correlated response must populate the same title cache as
        // updateNewChat or those messages degrade to an opaque sender.
        Some("chat") => payload,
        _ => return Ok(None),
    };
    if chat_payload
        .get("title")
        .and_then(Value::as_str)
        .is_none_or(|title| title.trim().is_empty())
    {
        return Ok(None);
    }
    parse_chat(account_id, chat_payload).map(Some)
}

fn private_chat_provider_user_id(payload: &Value) -> Option<String> {
    let chat = match payload.get("@type").and_then(Value::as_str) {
        Some("updateNewChat") => payload.get("chat")?,
        Some("chat") => payload,
        _ => return None,
    };
    let chat_type = chat.get("type")?;
    (chat_type.get("@type").and_then(Value::as_str) == Some("chatTypePrivate"))
        .then(|| chat_type.get("user_id").and_then(value_id_optional))
        .flatten()
}

fn parse_chat_avatar(account_id: &str, payload: &Value) -> Result<TelegramChatAvatar, TdlibError> {
    let photo = payload.get("photo").filter(|value| !value.is_null());
    let file = photo.and_then(|value| value.get("small").or_else(|| value.get("big")));
    Ok(TelegramChatAvatar {
        account_id: account_id.to_owned(),
        provider_chat_id: required_string(payload, "chat_id")?,
        provider_file_id: file
            .and_then(|value| value.get("id"))
            .and_then(value_id_optional),
        provider_unique_id: file
            .and_then(|value| value.get("remote"))
            .and_then(|value| value.get("unique_id"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

pub fn parse_message_observation(
    account_id: &str,
    payload: &Value,
) -> Result<TelegramMessageObservation, TdlibError> {
    let provider_chat_id = required_string(payload, "chat_id")?;
    let provider_message_id = required_string(payload, "id")?;
    let sender = message_sender_key(payload)
        .ok_or_else(|| TdlibError::Protocol("TDLib message sender is missing".to_owned()))?;
    let text = message_text(payload.get("content"));
    let media = parse_message_media(payload.get("content"));
    let references = parse_message_references(payload)?;
    Ok(TelegramMessageObservation {
        account_id: account_id.to_owned(),
        provider_chat_id,
        provider_message_id,
        provider_topic_id: payload.get("message_thread_id").and_then(value_id_optional),
        sender_id: sender.provider_id().to_owned(),
        sender_display_name: match &sender {
            TdlibMessageSenderKey::Chat(_) => {
                bounded_sender_display_name(payload.get("author_signature"))
                    .or_else(|| bounded_sender_display_name(payload.get("sender_display_name")))
            }
            TdlibMessageSenderKey::User(_) => {
                bounded_sender_display_name(payload.get("sender_display_name"))
            }
        }
        .or_else(|| Some(sender.fallback_display_name().to_owned())),
        sender_source_identity: Some(telegram_person_source_identity_v1(
            account_id,
            &sender.provider_source_key(),
        )),
        is_outgoing: payload
            .get("is_outgoing")
            .and_then(Value::as_bool)
            .ok_or_else(|| TdlibError::Protocol("TDLib message direction is missing".to_owned()))?,
        text,
        media,
        references,
        observed_at_unix_seconds: payload
            .get("date")
            .and_then(Value::as_i64)
            .unwrap_or_default(),
    })
}

fn parse_message_references(payload: &Value) -> Result<TelegramMessageReferences, TdlibError> {
    let reply_to = match payload.get("reply_to") {
        None => None,
        Some(reply) => {
            if reply.get("@type").and_then(Value::as_str) != Some("messageReplyToMessage") {
                return Err(TdlibError::Protocol(
                    "TDLib reply reference type is unsupported".to_owned(),
                ));
            }
            Some(TelegramReplyReference {
                provider_chat_id: required_string(reply, "chat_id")?,
                provider_message_id: required_string(reply, "message_id")?,
            })
        }
    };
    let forward_origin = match payload.get("forward_info") {
        None => None,
        Some(forward_info) => {
            let origin = forward_info.get("origin").ok_or_else(|| {
                TdlibError::Protocol("TDLib forward origin is missing".to_owned())
            })?;
            let origin_type = origin.get("@type").and_then(Value::as_str).ok_or_else(|| {
                TdlibError::Protocol("TDLib forward origin type is missing".to_owned())
            })?;
            let provider_sender_id = match origin_type {
                "messageOriginUser" | "messageForwardOriginUser" => {
                    Some(value_id(origin.get("sender_user_id").ok_or_else(
                        || TdlibError::Protocol("TDLib forward user origin is missing".to_owned()),
                    )?)?)
                }
                "messageOriginChat" | "messageForwardOriginChat" => {
                    Some(value_id(origin.get("sender_chat_id").ok_or_else(
                        || TdlibError::Protocol("TDLib forward chat origin is missing".to_owned()),
                    )?)?)
                }
                "messageOriginChannel" | "messageForwardOriginChannel" => {
                    Some(value_id(origin.get("chat_id").ok_or_else(|| {
                        TdlibError::Protocol("TDLib forward channel origin is missing".to_owned())
                    })?)?)
                }
                "messageOriginHiddenUser"
                | "messageOriginMessageImport"
                | "messageForwardOriginHiddenUser"
                | "messageForwardOriginMessageImport" => None,
                other => {
                    return Err(TdlibError::Protocol(format!(
                        "TDLib forward origin type is unsupported: {other}"
                    )));
                }
            };
            let sender_name = origin
                .get("sender_name")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let provider_chat_id = forward_info.get("chat_id").map(value_id).transpose()?;
            let provider_message_id = forward_info.get("message_id").map(value_id).transpose()?;
            Some(TelegramForwardOrigin {
                provider_chat_id,
                provider_message_id,
                provider_sender_id,
                sender_name,
                observed_at_unix_seconds: forward_info.get("date").and_then(Value::as_i64),
            })
        }
    };
    Ok(TelegramMessageReferences {
        reply_to,
        forward_origin,
    })
}

pub fn parse_file_snapshot(
    account_id: &str,
    payload: &Value,
) -> Result<TelegramFileSnapshot, TdlibError> {
    if payload.get("@type").and_then(Value::as_str) != Some("file") {
        return Err(TdlibError::Protocol(
            "TDLib file payload is invalid".to_owned(),
        ));
    }
    let provider_file_id = required_string(payload, "id")?;
    let local = payload.get("local");
    let remote = payload.get("remote");
    Ok(TelegramFileSnapshot {
        account_id: account_id.to_owned(),
        provider_file_id,
        provider_unique_id: remote
            .and_then(|value| value.get("unique_id"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        media_kind: None,
        size_bytes: integer_field(payload, "size"),
        expected_size_bytes: integer_field(payload, "expected_size"),
        downloaded_size_bytes: local.and_then(|value| integer_field(value, "downloaded_size")),
        is_downloading: local
            .and_then(|value| value.get("is_downloading_active"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        is_downloaded: local
            .and_then(|value| value.get("is_downloading_completed"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        blob_reference_id: None,
        blob_plaintext_sha256: None,
        blob_backup_class: None,
    })
}

fn downloaded_file(
    account_id: &str,
    payload: &Value,
) -> Result<Option<TdlibDownloadedFile>, TdlibError> {
    if payload.get("@type").and_then(Value::as_str) != Some("updateFile") {
        return Ok(None);
    }
    let file = payload
        .get("file")
        .ok_or_else(|| TdlibError::Protocol("updateFile has no file".to_owned()))?;
    let snapshot = parse_file_snapshot(account_id, file)?;
    if !snapshot.is_downloaded {
        return Ok(None);
    }
    let local_path = file
        .get("local")
        .and_then(|value| value.get("path"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| {
            TdlibError::Protocol("Downloaded TDLib file has no local path".to_owned())
        })?;
    Ok(Some(TdlibDownloadedFile {
        snapshot,
        local_path,
    }))
}

fn completed_download_response_update(
    account_id: &str,
    payload: &Value,
) -> Result<Option<Value>, TdlibError> {
    if payload.get("@type").and_then(Value::as_str) != Some("file") {
        return Ok(None);
    }
    let update = json!({"@type": "updateFile", "file": payload});
    downloaded_file(account_id, &update).map(|downloaded| downloaded.map(|_| update))
}

fn is_download_file_request(request: &TdlibRequest) -> bool {
    matches!(
        request,
        TdlibRequest::DownloadFile(_)
            | TdlibRequest::ProviderCommand(TelegramProviderCommand::DownloadFile(_))
    )
}

pub fn parse_participant_page(
    account_id: &str,
    provider_chat_id: &str,
    filter: TelegramParticipantFilter,
    offset: u32,
    payload: &Value,
) -> Result<TelegramParticipantPage, TdlibError> {
    let members = payload
        .get("members")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TdlibError::Protocol("TDLib participant list is missing members".to_owned())
        })?;
    let items = members
        .iter()
        .map(|member| {
            let member_id = member.get("member_id").ok_or_else(|| {
                TdlibError::Protocol("TDLib participant id is missing".to_owned())
            })?;
            let (member_kind, member_id) = if let Some(user_id) = member_id.get("user_id") {
                ("user", value_id(user_id)?)
            } else if let Some(chat_id) = member_id.get("chat_id") {
                ("chat", value_id(chat_id)?)
            } else {
                return Err(TdlibError::Protocol(
                    "TDLib participant sender kind is unsupported".to_owned(),
                ));
            };
            let status_kind = member
                .get("status")
                .and_then(|value| value.get("@type"))
                .and_then(Value::as_str)
                .unwrap_or("chatMemberStatusUnknown")
                .to_owned();
            let is_admin = matches!(
                status_kind.as_str(),
                "chatMemberStatusAdministrator" | "chatMemberStatusCreator"
            );
            let is_owner = status_kind == "chatMemberStatusCreator";
            Ok(TelegramParticipant {
                account_id: account_id.to_owned(),
                provider_chat_id: provider_chat_id.to_owned(),
                provider_member_id: format!("{member_kind}:{member_id}"),
                display_name: optional_trimmed(member.get("display_name")),
                username: optional_trimmed(member.get("username")),
                role: participant_role(&status_kind).to_owned(),
                status: status_kind
                    .strip_prefix("chatMemberStatus")
                    .unwrap_or(&status_kind)
                    .to_lowercase(),
                is_admin,
                is_owner,
                permissions: participant_permissions(member.get("status")),
            })
        })
        .collect::<Result<Vec<_>, TdlibError>>()?;
    let next_offset = (!items.is_empty()).then_some(offset + items.len() as u32);
    Ok(TelegramParticipantPage {
        account_id: account_id.to_owned(),
        provider_chat_id: provider_chat_id.to_owned(),
        filter,
        items,
        next_offset,
    })
}

pub fn parse_topic_list(
    account_id: &str,
    provider_chat_id: &str,
    payload: &Value,
) -> Result<Vec<TelegramTopic>, TdlibError> {
    payload
        .get("topics")
        .and_then(Value::as_array)
        .ok_or_else(|| TdlibError::Protocol("TDLib topic list is missing topics".to_owned()))?
        .iter()
        .map(|topic| {
            let info = topic.get("info").unwrap_or(topic);
            let mut parsed = parse_topic_info(account_id, provider_chat_id, info)?;
            parsed.unread_count = topic
                .get("unread_count")
                .and_then(Value::as_i64)
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or_default();
            parsed.last_message_at_unix_seconds = topic
                .get("last_message")
                .and_then(|message| message.get("date"))
                .and_then(Value::as_i64);
            Ok(parsed)
        })
        .collect()
}

fn parse_topic_info(
    account_id: &str,
    provider_chat_id: &str,
    info: &Value,
) -> Result<TelegramTopic, TdlibError> {
    let provider_topic_id = required_string(info, "message_thread_id")?;
    Ok(TelegramTopic {
        account_id: account_id.to_owned(),
        provider_chat_id: provider_chat_id.to_owned(),
        provider_topic_id,
        title: info
            .get("name")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("Telegram topic")
            .to_owned(),
        icon_emoji: info
            .get("icon")
            .and_then(|icon| icon.get("custom_emoji_id"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty() && *value != "0")
            .map(ToOwned::to_owned),
        is_pinned: info
            .get("is_pinned")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        is_closed: info
            .get("is_closed")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        unread_count: 0,
        last_message_at_unix_seconds: None,
    })
}

fn parse_typing_state(
    account_id: &str,
    payload: &Value,
) -> Result<TelegramTypingState, TdlibError> {
    let sender = payload
        .get("sender_id")
        .ok_or_else(|| TdlibError::Protocol("TDLib typing sender is missing".to_owned()))?;
    let sender_id = match sender.get("@type").and_then(Value::as_str) {
        Some("messageSenderUser") => format!("user:{}", required_string(sender, "user_id")?),
        Some("messageSenderChat") => format!("chat:{}", required_string(sender, "chat_id")?),
        _ => {
            return Err(TdlibError::Protocol(
                "TDLib typing sender kind is unsupported".to_owned(),
            ));
        }
    };
    let action = payload
        .get("action")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib typing action is missing".to_owned()))?;
    Ok(TelegramTypingState {
        account_id: account_id.to_owned(),
        provider_chat_id: required_string(payload, "chat_id")?,
        provider_thread_id: payload.get("message_thread_id").and_then(value_id_optional),
        sender_id,
        action: action.to_owned(),
        is_active: action != "chatActionCancel",
    })
}

fn parse_chat_position(
    account_id: &str,
    payload: &Value,
) -> Result<Option<TelegramChatPosition>, TdlibError> {
    let position = payload
        .get("position")
        .ok_or_else(|| TdlibError::Protocol("TDLib chat position is missing".to_owned()))?;
    let list = position
        .get("list")
        .ok_or_else(|| TdlibError::Protocol("TDLib chat position list is missing".to_owned()))?;
    let list_type = list
        .get("@type")
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib chat list type is missing".to_owned()))?;
    let (list_kind, provider_folder_id) = match list_type {
        "chatListMain" => ("main".to_owned(), None),
        "chatListArchive" => ("archive".to_owned(), None),
        "chatListFolder" => (
            "folder".to_owned(),
            list.get("chat_folder_id").and_then(Value::as_i64),
        ),
        _ => return Ok(None),
    };
    Ok(Some(TelegramChatPosition {
        account_id: account_id.to_owned(),
        provider_chat_id: required_string(payload, "chat_id")?,
        list_kind,
        provider_folder_id,
        order: position
            .get("order")
            .and_then(Value::as_i64)
            .unwrap_or_default(),
        is_pinned: position
            .get("is_pinned")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }))
}

fn parse_chat_folders(
    account_id: &str,
    payload: &Value,
) -> Result<Vec<TelegramChatFolder>, TdlibError> {
    payload
        .get("chat_folders")
        .and_then(Value::as_array)
        .ok_or_else(|| TdlibError::Protocol("TDLib chat folders are missing".to_owned()))?
        .iter()
        .map(|folder| {
            Ok(TelegramChatFolder {
                account_id: account_id.to_owned(),
                provider_folder_id: required_string(folder, "id")?.parse().map_err(|_| {
                    TdlibError::Protocol("Telegram folder id is invalid".to_owned())
                })?,
                title: folder
                    .get("name")
                    .and_then(|name| name.get("text"))
                    .and_then(Value::as_str)
                    .unwrap_or("Telegram folder")
                    .to_owned(),
                icon_name: folder
                    .get("icon")
                    .and_then(|icon| icon.get("name"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                color_id: folder.get("color_id").and_then(Value::as_i64),
                pinned_chat_ids: folder_id_list(folder, "pinned_chat_ids")?,
                included_chat_ids: folder_id_list(folder, "included_chat_ids")?,
                excluded_chat_ids: folder_id_list(folder, "excluded_chat_ids")?,
            })
        })
        .collect()
}

fn folder_id_list(folder: &Value, field: &str) -> Result<Vec<String>, TdlibError> {
    let Some(values) = folder.get(field).and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    values
        .iter()
        .map(|value| {
            value
                .as_i64()
                .map(|id| id.to_string())
                .or_else(|| value.as_str().map(ToOwned::to_owned))
                .ok_or_else(|| {
                    TdlibError::Protocol("Telegram folder chat id is invalid".to_owned())
                })
        })
        .collect()
}

pub fn parse_provider_events(
    account_id: &str,
    payload: &Value,
) -> Result<Vec<TelegramProviderEvent>, TdlibError> {
    let event_type = payload
        .get("@type")
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib update type is missing".to_owned()))?;
    let event = match event_type {
        "updateNewMessage" => {
            let message = payload.get("message").ok_or_else(|| {
                TdlibError::Protocol("updateNewMessage has no message".to_owned())
            })?;
            TelegramProviderEvent::MessageCreated(parse_message_observation(account_id, message)?)
        }
        "updateUserChatAction" => {
            TelegramProviderEvent::TypingChanged(parse_typing_state(account_id, payload)?)
        }
        "updateForumTopicInfo" => {
            let chat_id = required_string(payload, "chat_id")?;
            let info = payload.get("info").ok_or_else(|| {
                TdlibError::Protocol("updateForumTopicInfo has no info".to_owned())
            })?;
            TelegramProviderEvent::TopicChanged(parse_topic_info(account_id, &chat_id, info)?)
        }
        "updateMessageSendFailed" => TelegramProviderEvent::MessageSendFailed {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            old_provider_message_id: required_string(payload, "old_message_id")?,
            error_code: payload
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(Value::as_i64),
        },
        "updateMessageSendSucceeded" => {
            let message = payload.get("message").ok_or_else(|| {
                TdlibError::Protocol("updateMessageSendSucceeded has no message".to_owned())
            })?;
            TelegramProviderEvent::MessageSendSucceeded {
                account_id: account_id.to_owned(),
                provider_chat_id: required_string(payload, "chat_id")?,
                old_provider_message_id: required_string(payload, "old_message_id")?,
                provider_message_id: required_string(message, "id")?,
            }
        }
        "updateChatPosition" => {
            let Some(position) = parse_chat_position(account_id, payload)? else {
                return Ok(Vec::new());
            };
            TelegramProviderEvent::ChatPositionChanged(position)
        }
        "updateChatFolders" => TelegramProviderEvent::ChatFoldersChanged {
            account_id: account_id.to_owned(),
            folders: parse_chat_folders(account_id, payload)?,
        },
        "updateChatNotificationSettings" => {
            let settings = payload.get("notification_settings").ok_or_else(|| {
                TdlibError::Protocol("TDLib notification settings are missing".to_owned())
            })?;
            TelegramProviderEvent::ChatNotificationChanged {
                account_id: account_id.to_owned(),
                provider_chat_id: required_string(payload, "chat_id")?,
                use_default_mute_for: settings
                    .get("use_default_mute_for")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
                mute_for_seconds: settings
                    .get("mute_for")
                    .and_then(Value::as_i64)
                    .unwrap_or_default(),
            }
        }
        "updateChatPhoto" => {
            TelegramProviderEvent::ChatAvatarChanged(parse_chat_avatar(account_id, payload)?)
        }
        "updateChatMember" => {
            let provider_chat_id = required_string(payload, "chat_id")?;
            let member = payload.get("new_chat_member").ok_or_else(|| {
                TdlibError::Protocol("updateChatMember has no new_chat_member".to_owned())
            })?;
            let page = parse_participant_page(
                account_id,
                &provider_chat_id,
                TelegramParticipantFilter::Recent,
                0,
                &json!({"members": [member]}),
            )?;
            let participant = page.items.into_iter().next().ok_or_else(|| {
                TdlibError::Protocol("updateChatMember has no participant".to_owned())
            })?;
            TelegramProviderEvent::ParticipantChanged(participant)
        }
        "updateFile" => TelegramProviderEvent::FileChanged(parse_file_snapshot(
            account_id,
            payload
                .get("file")
                .ok_or_else(|| TdlibError::Protocol("updateFile has no file".to_owned()))?,
        )?),
        "updateMessageContent" => TelegramProviderEvent::MessageEdited {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            provider_message_id: required_string(payload, "message_id")?,
            text: message_text(payload.get("new_content")),
            observed_at_unix_seconds: 0,
        },
        "updateMessageEdited" => TelegramProviderEvent::MessageEdited {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            provider_message_id: required_string(payload, "message_id")?,
            text: None,
            observed_at_unix_seconds: payload
                .get("edit_date")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
        },
        "updateMessageIsPinned" => TelegramProviderEvent::MessagePinned {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            provider_message_id: required_string(payload, "message_id")?,
            is_pinned: payload
                .get("is_pinned")
                .and_then(Value::as_bool)
                .ok_or_else(|| TdlibError::Protocol("pinned state is missing".to_owned()))?,
        },
        "updateMessageInteractionInfo" => TelegramProviderEvent::ReactionsObserved {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            provider_message_id: required_string(payload, "message_id")?,
            reactions: parse_reaction_observations(payload)?,
        },
        "updateDeleteMessages" => {
            let chat_id = required_string(payload, "chat_id")?;
            let is_permanent = payload
                .get("is_permanent")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let message_ids = payload
                .get("message_ids")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    TdlibError::Protocol("deleted message ids are missing".to_owned())
                })?;
            return message_ids
                .iter()
                .map(|message_id| {
                    Ok(TelegramProviderEvent::MessageDeleted {
                        account_id: account_id.to_owned(),
                        provider_chat_id: chat_id.clone(),
                        provider_message_id: value_id(message_id)?,
                        is_permanent,
                    })
                })
                .collect();
        }
        "updateChatReadInbox" => TelegramProviderEvent::ChatUnreadChanged {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            unread_count: payload.get("unread_count").and_then(Value::as_i64),
            unread_mention_count: None,
            last_read_inbox_message_id: payload
                .get("last_read_inbox_message_id")
                .map(value_id)
                .transpose()?,
        },
        "updateChatUnreadMentionCount" => TelegramProviderEvent::ChatUnreadChanged {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            unread_count: None,
            unread_mention_count: payload.get("unread_mention_count").and_then(Value::as_i64),
            last_read_inbox_message_id: None,
        },
        "updateChatIsMarkedAsUnread" => TelegramProviderEvent::ChatMarkedUnreadChanged {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            is_marked_as_unread: payload
                .get("is_marked_as_unread")
                .and_then(Value::as_bool)
                .ok_or_else(|| TdlibError::Protocol("marked unread state is missing".to_owned()))?,
        },
        _ => return Ok(Vec::new()),
    };
    Ok(vec![event])
}

pub fn parse_provider_updates(
    account_id: &str,
    payload: &Value,
) -> Result<Vec<TdlibProviderUpdate>, TdlibError> {
    if payload.get("@type").and_then(Value::as_str) == Some("updateNewCallSignalingData") {
        let tdlib_call_id = integer_field(payload, "call_id")
            .and_then(|value| i32::try_from(value).ok())
            .filter(|value| *value > 0)
            .ok_or_else(|| TdlibError::Protocol("TDLib call id is invalid".to_owned()))?;
        let encoded = payload.get("data").and_then(Value::as_str).ok_or_else(|| {
            TdlibError::Protocol("TDLib call signaling data is missing".to_owned())
        })?;
        let data = STANDARD
            .decode(encoded)
            .map_err(|_| TdlibError::Protocol("TDLib call signaling data is invalid".to_owned()))?;
        return Ok(vec![TdlibProviderUpdate::CallSignaling {
            account_id: account_id.to_owned(),
            tdlib_call_id,
            data: TelegramCallSecretBytesV1::new(data, MAX_SIGNALING_DATA_BYTES).map_err(|_| {
                TdlibError::Protocol("TDLib call signaling data is invalid".to_owned())
            })?,
        }]);
    }
    if payload.get("@type").and_then(Value::as_str) == Some("updateCall") {
        let observation = parse_call_observation(account_id, payload)?;
        if observation.state == TdlibCallState::Ready {
            let state = payload
                .get("call")
                .and_then(|call| call.get("state"))
                .ok_or_else(|| TdlibError::Protocol("TDLib call state is missing".to_owned()))?;
            let material = parse_call_ready_material(
                state,
                observation.direction == TdlibCallDirection::Outgoing,
            )?;
            return Ok(vec![TdlibProviderUpdate::CallReady {
                observation,
                material,
            }]);
        }
        return Ok(vec![TdlibProviderUpdate::Call(observation)]);
    }
    let downloaded = downloaded_file(account_id, payload)?;
    parse_provider_events(account_id, payload).map(|events| {
        let mut updates = events
            .into_iter()
            .map(|event| TdlibProviderUpdate::Operational(Box::new(event)))
            .collect::<Vec<_>>();
        if let Some(downloaded) = downloaded {
            updates.push(TdlibProviderUpdate::DownloadedFile(downloaded));
        }
        updates
    })
}

fn parse_call_ready_material(
    state: &Value,
    is_outgoing: bool,
) -> Result<TelegramCallReadyMaterialV1, TdlibError> {
    let peer_protocol = parse_call_peer_protocol(
        state
            .get("protocol")
            .ok_or_else(|| TdlibError::Protocol("TDLib call protocol is missing".to_owned()))?,
    )?;
    let server_values = state
        .get("servers")
        .and_then(Value::as_array)
        .filter(|servers| !servers.is_empty())
        .ok_or_else(|| TdlibError::Protocol("TDLib call servers are missing".to_owned()))?;
    let mut servers = Vec::with_capacity(server_values.len());
    let mut allow_tcp = false;
    for server in server_values {
        let ipv4 = server
            .get("ip_address")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let ipv6 = server
            .get("ipv6_address")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let port = integer_field(server, "port")
            .and_then(|value| u16::try_from(value).ok())
            .filter(|value| *value > 0)
            .ok_or_else(|| TdlibError::Protocol("TDLib call server port is invalid".to_owned()))?;
        let server_type = server
            .get("type")
            .ok_or_else(|| TdlibError::Protocol("TDLib call server type is missing".to_owned()))?;
        let kind = match server_type.get("@type").and_then(Value::as_str) {
            Some("callServerTypeTelegramReflector") => {
                let reflector_id = integer_field(server, "id")
                    .and_then(|value| u8::try_from(value).ok())
                    .ok_or_else(|| {
                        TdlibError::Protocol("TDLib reflector id is unsupported".to_owned())
                    })?;
                let peer_tag = server_type
                    .get("peer_tag")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        TdlibError::Protocol("TDLib reflector peer tag is missing".to_owned())
                    })
                    .and_then(|encoded| {
                        STANDARD.decode(encoded).map_err(|_| {
                            TdlibError::Protocol("TDLib reflector peer tag is invalid".to_owned())
                        })
                    })?;
                let peer_tag: [u8; 16] = peer_tag.try_into().map_err(|_| {
                    TdlibError::Protocol("TDLib reflector peer tag is invalid".to_owned())
                })?;
                let is_tcp = server_type
                    .get("is_tcp")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| {
                        TdlibError::Protocol("TDLib reflector transport is missing".to_owned())
                    })?;
                allow_tcp |= is_tcp;
                TelegramCallServerKindV1::TelegramReflector {
                    reflector_id,
                    peer_tag,
                    is_tcp,
                }
            }
            Some("callServerTypeWebrtc") => {
                let supports_stun = server_type
                    .get("supports_stun")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| {
                        TdlibError::Protocol("TDLib WebRTC STUN flag is missing".to_owned())
                    })?;
                let supports_turn = server_type
                    .get("supports_turn")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| {
                        TdlibError::Protocol("TDLib WebRTC TURN flag is missing".to_owned())
                    })?;
                allow_tcp |= supports_turn;
                TelegramCallServerKindV1::WebRtc {
                    username: TelegramCallSecretTextV1::new(
                        server_type
                            .get("username")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        MAX_SERVER_CREDENTIAL_BYTES,
                    )
                    .map_err(|_| {
                        TdlibError::Protocol("TDLib WebRTC username is invalid".to_owned())
                    })?,
                    password: TelegramCallSecretTextV1::new(
                        server_type
                            .get("password")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        MAX_SERVER_CREDENTIAL_BYTES,
                    )
                    .map_err(|_| {
                        TdlibError::Protocol("TDLib WebRTC password is invalid".to_owned())
                    })?,
                    supports_stun,
                    supports_turn,
                }
            }
            _ => {
                return Err(TdlibError::Protocol(
                    "TDLib call server type is unsupported".to_owned(),
                ));
            }
        };
        servers.push(TelegramCallServerV1 {
            ipv4,
            ipv6,
            port,
            kind,
        });
    }
    let call_config = state
        .get("config")
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib call config is missing".to_owned()))?;
    let custom_parameters = state
        .get("custom_parameters")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            TdlibError::Protocol("TDLib call custom parameters are missing".to_owned())
        })?;
    let encryption_key = state
        .get("encryption_key")
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib call encryption key is missing".to_owned()))
        .and_then(|encoded| {
            STANDARD.decode(encoded).map_err(|_| {
                TdlibError::Protocol("TDLib call encryption key is invalid".to_owned())
            })
        })?;
    if encryption_key.len() != CALL_ENCRYPTION_KEY_BYTES {
        return Err(TdlibError::Protocol(
            "TDLib call encryption key is invalid".to_owned(),
        ));
    }
    Ok(TelegramCallReadyMaterialV1 {
        peer_protocol,
        servers,
        allow_p2p: state
            .get("allow_p2p")
            .and_then(Value::as_bool)
            .ok_or_else(|| TdlibError::Protocol("TDLib call P2P flag is missing".to_owned()))?,
        allow_tcp,
        call_config: TelegramCallSecretTextV1::new(call_config.to_owned(), MAX_READY_TEXT_BYTES)
            .map_err(|_| TdlibError::Protocol("TDLib call config is invalid".to_owned()))?,
        custom_parameters: TelegramCallSecretTextV1::new(
            custom_parameters.to_owned(),
            MAX_READY_TEXT_BYTES,
        )
        .map_err(|_| TdlibError::Protocol("TDLib call custom parameters are invalid".to_owned()))?,
        encryption_key: TelegramCallSecretBytesV1::new(encryption_key, CALL_ENCRYPTION_KEY_BYTES)
            .map_err(|_| {
            TdlibError::Protocol("TDLib call encryption key is invalid".to_owned())
        })?,
        is_outgoing,
    })
}

fn parse_call_peer_protocol(protocol: &Value) -> Result<TelegramCallPeerProtocolV1, TdlibError> {
    let library_versions = protocol
        .get("library_versions")
        .and_then(Value::as_array)
        .ok_or_else(|| TdlibError::Protocol("TDLib call library versions are missing".to_owned()))?
        .iter()
        .map(|version| {
            version.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                TdlibError::Protocol("TDLib call library version is invalid".to_owned())
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TelegramCallPeerProtocolV1 {
        udp_p2p: protocol
            .get("udp_p2p")
            .and_then(Value::as_bool)
            .ok_or_else(|| TdlibError::Protocol("TDLib call P2P protocol is missing".to_owned()))?,
        udp_reflector: protocol
            .get("udp_reflector")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                TdlibError::Protocol("TDLib call reflector protocol is missing".to_owned())
            })?,
        min_layer: protocol
            .get("min_layer")
            .and_then(Value::as_i64)
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| TdlibError::Protocol("TDLib call min layer is invalid".to_owned()))?,
        max_layer: protocol
            .get("max_layer")
            .and_then(Value::as_i64)
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| TdlibError::Protocol("TDLib call max layer is invalid".to_owned()))?,
        library_versions,
    })
}

fn parse_call_observation(
    account_id: &str,
    payload: &Value,
) -> Result<TdlibCallObservation, TdlibError> {
    let call = payload
        .get("call")
        .ok_or_else(|| TdlibError::Protocol("updateCall has no call".to_owned()))?;
    let tdlib_call_id = integer_field(call, "id")
        .and_then(|value| i32::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| TdlibError::Protocol("TDLib call id is invalid".to_owned()))?;
    let provider_call_unique_id = integer_field(call, "unique_id")
        .filter(|value| *value > 0)
        .map(i64::try_from)
        .transpose()
        .map_err(|_| TdlibError::Protocol("TDLib persistent call id is invalid".to_owned()))?;
    let provider_user_id = required_string(call, "user_id")?;
    let direction = if call
        .get("is_outgoing")
        .and_then(Value::as_bool)
        .ok_or_else(|| TdlibError::Protocol("TDLib call direction is missing".to_owned()))?
    {
        TdlibCallDirection::Outgoing
    } else {
        TdlibCallDirection::Incoming
    };
    let is_video = call
        .get("is_video")
        .and_then(Value::as_bool)
        .ok_or_else(|| TdlibError::Protocol("TDLib call media kind is missing".to_owned()))?;
    let state = call
        .get("state")
        .ok_or_else(|| TdlibError::Protocol("TDLib call state is missing".to_owned()))?;
    let state_type = state
        .get("@type")
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib call state type is missing".to_owned()))?;

    let (normalized_state, pending_created, pending_received, discard_reason, failure_category) =
        match state_type {
            "callStatePending" => (
                TdlibCallState::Pending,
                state
                    .get("is_created")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                state
                    .get("is_received")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                None,
                None,
            ),
            "callStateExchangingKeys" => (TdlibCallState::ExchangingKeys, false, false, None, None),
            "callStateReady" => (TdlibCallState::Ready, false, false, None, None),
            "callStateHangingUp" => (TdlibCallState::HangingUp, false, false, None, None),
            "callStateDiscarded" => (
                TdlibCallState::Discarded,
                false,
                false,
                Some(parse_call_discard_reason(state)?),
                None,
            ),
            "callStateError" => (
                TdlibCallState::Error,
                false,
                false,
                None,
                Some(call_failure_category(state)),
            ),
            _ => {
                return Err(TdlibError::Protocol(
                    "TDLib call state is unsupported".to_owned(),
                ));
            }
        };

    Ok(TdlibCallObservation {
        account_id: account_id.to_owned(),
        tdlib_call_id,
        provider_call_unique_id,
        provider_user_id,
        direction,
        is_video,
        state: normalized_state,
        pending_created,
        pending_received,
        discard_reason,
        failure_category,
    })
}

fn parse_call_discard_reason(payload: &Value) -> Result<TdlibCallDiscardReason, TdlibError> {
    let reason_type = payload
        .get("reason")
        .and_then(|reason| reason.get("@type"))
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib call discard reason is missing".to_owned()))?;
    match reason_type {
        "callDiscardReasonEmpty" => Ok(TdlibCallDiscardReason::Empty),
        "callDiscardReasonMissed" => Ok(TdlibCallDiscardReason::Missed),
        "callDiscardReasonDeclined" => Ok(TdlibCallDiscardReason::Declined),
        "callDiscardReasonDisconnected" => Ok(TdlibCallDiscardReason::Disconnected),
        "callDiscardReasonHungUp" => Ok(TdlibCallDiscardReason::HungUp),
        _ => Err(TdlibError::Protocol(
            "TDLib call discard reason is unsupported".to_owned(),
        )),
    }
}

fn call_failure_category(payload: &Value) -> TdlibCallFailureCategory {
    match payload
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(Value::as_i64)
    {
        Some(401 | 403) => TdlibCallFailureCategory::Permission,
        Some(404) => TdlibCallFailureCategory::NotAvailable,
        Some(408 | 429 | 500..=599) => TdlibCallFailureCategory::Network,
        _ => TdlibCallFailureCategory::Unknown,
    }
}

fn message_text(content: Option<&Value>) -> Option<String> {
    let content = content?;
    content
        .get("text")
        .or_else(|| content.get("caption"))
        .and_then(|value| value.get("text"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| message_content_summary(content))
}

fn message_content_summary(content: &Value) -> Option<String> {
    let content_type = content.get("@type").and_then(Value::as_str)?;
    match content_type {
        "messageText" => Some("Text message".to_owned()),
        "messageRichMessage" => Some("Rich message".to_owned()),
        "messageAnimatedEmoji" => {
            summary_with_detail("Animated emoji", trimmed_string(content.get("emoji")))
        }
        "messagePhoto" => Some("Photo".to_owned()),
        "messageVideo" => Some("Video".to_owned()),
        "messageAudio" => Some("Audio".to_owned()),
        "messageDocument" => Some("File".to_owned()),
        "messageAnimation" => Some("Animation".to_owned()),
        "messageVoiceNote" => Some("Voice message".to_owned()),
        "messageSticker" => content
            .get("sticker")
            .and_then(|sticker| trimmed_string(sticker.get("emoji")))
            .or_else(|| Some("Sticker".to_owned())),
        "messageVideoNote" => Some("Video message".to_owned()),
        "messagePaidMedia" => Some("Paid media".to_owned()),
        "messagePoll" => summary_with_detail(
            "Poll",
            nested_formatted_text(content, &["poll", "question"]),
        ),
        "messagePollOptionAdded" => Some("Poll option added".to_owned()),
        "messagePollOptionDeleted" => Some("Poll option removed".to_owned()),
        "messageContact" => content
            .get("contact")
            .and_then(contact_summary)
            .or_else(|| Some("Contact".to_owned())),
        "messageLocation" => content
            .get("location")
            .and_then(location_summary)
            .or_else(|| Some("Location".to_owned())),
        "messageLiveLocation" => content
            .get("location")
            .and_then(location_summary)
            .map(|value| value.replacen("Location", "Live location", 1))
            .or_else(|| Some("Live location".to_owned())),
        "messageVenue" => content
            .get("venue")
            .and_then(venue_summary)
            .or_else(|| Some("Venue".to_owned())),
        "messageDice" | "messageStakeDice" => dice_summary(content),
        "messageGame" => summary_with_detail(
            "Game",
            trimmed_string(content.get("game").and_then(|game| game.get("title"))),
        ),
        "messageGameScore" => Some("Game score".to_owned()),
        "messageInvoice" => summary_with_detail(
            "Invoice",
            nested_formatted_text(content, &["product_info", "title"]),
        ),
        "messageChecklist" => summary_with_detail(
            "Checklist",
            nested_formatted_text(content, &["list", "title"]),
        ),
        "messageChecklistTasksAdded" => Some("Checklist tasks added".to_owned()),
        "messageChecklistTasksDone" => Some("Checklist tasks updated".to_owned()),
        "messageBasicGroupChatCreate" | "messageSupergroupChatCreate" => {
            summary_with_detail("Group created", trimmed_string(content.get("title")))
        }
        "messageChatChangeTitle" => {
            summary_with_detail("Chat title changed", trimmed_string(content.get("title")))
        }
        "messageCustomServiceAction" => {
            summary_with_detail("Service message", trimmed_string(content.get("text")))
        }
        "messageExpiredPhoto" => Some("Expired photo".to_owned()),
        "messageExpiredVideo" => Some("Expired video".to_owned()),
        "messageExpiredVideoNote" => Some("Expired video message".to_owned()),
        "messageExpiredVoiceNote" => Some("Expired voice message".to_owned()),
        "messageStory" => Some("Story".to_owned()),
        "messageCall" => Some("Call".to_owned()),
        "messageGroupCall" => Some("Group call".to_owned()),
        "messageVideoChatScheduled" => Some("Video chat scheduled".to_owned()),
        "messageVideoChatStarted" => Some("Video chat started".to_owned()),
        "messageVideoChatEnded" => Some("Video chat ended".to_owned()),
        "messageChatAddMembers" => Some("Members added".to_owned()),
        "messageChatDeleteMember" => Some("Member removed".to_owned()),
        "messageChatJoinByLink" => Some("Joined via invite link".to_owned()),
        "messageChatJoinByRequest" => Some("Join request approved".to_owned()),
        "messagePinMessage" => Some("Message pinned".to_owned()),
        "messageScreenshotTaken" => Some("Screenshot taken".to_owned()),
        "messageChatChangePhoto" => Some("Chat photo changed".to_owned()),
        "messageChatDeletePhoto" => Some("Chat photo removed".to_owned()),
        "messageBotWriteAccessAllowed" => Some("Bot write access allowed".to_owned()),
        "messageChatAddedToCommunity" => Some("Chat added to community".to_owned()),
        "messageChatRemovedFromCommunity" => Some("Chat removed from community".to_owned()),
        "messageChatBoost" => Some("Chat boosted".to_owned()),
        "messageChatHasProtectedContentDisableRequested" => {
            Some("Protected content disable requested".to_owned())
        }
        "messageChatHasProtectedContentToggled" => {
            Some("Protected content setting changed".to_owned())
        }
        "messageChatOwnerChanged" => Some("Chat owner changed".to_owned()),
        "messageChatOwnerLeft" => Some("Chat owner left".to_owned()),
        "messageChatSetBackground" => Some("Chat background changed".to_owned()),
        "messageChatSetMessageAutoDeleteTime" => {
            Some("Message auto-delete timer changed".to_owned())
        }
        "messageChatSetTheme" => Some("Chat theme changed".to_owned()),
        "messageChatSetTtl" => Some("Message auto-delete timer changed".to_owned()),
        "messageChatShared" => Some("Chat shared".to_owned()),
        "messageChatUpgradeFrom" | "messageChatUpgradeTo" => Some("Chat upgraded".to_owned()),
        "messageDirectMessagePriceChanged" => Some("Direct message price changed".to_owned()),
        "messageContactRegistered" => Some("Contact joined Telegram".to_owned()),
        "messageForumTopicCreated" => Some("Forum topic created".to_owned()),
        "messageForumTopicEdited" => Some("Forum topic edited".to_owned()),
        "messageForumTopicIsClosedToggled" => Some("Forum topic state changed".to_owned()),
        "messageForumTopicIsHiddenToggled" => Some("Forum topic visibility changed".to_owned()),
        "messageGift" => Some("Gift".to_owned()),
        "messageGiftedPremium" => Some("Telegram Premium gift".to_owned()),
        "messageGiftedStars" => Some("Telegram Stars gift".to_owned()),
        "messageGiftedTon" => Some("TON gift".to_owned()),
        "messageGiveaway" => Some("Giveaway".to_owned()),
        "messageGiveawayCompleted" => Some("Giveaway completed".to_owned()),
        "messageGiveawayCreated" => Some("Giveaway created".to_owned()),
        "messageGiveawayPrizeStars" => Some("Giveaway prize".to_owned()),
        "messageGiveawayWinners" => Some("Giveaway winners".to_owned()),
        "messageInviteVideoChatParticipants" => Some("Video chat participants invited".to_owned()),
        "messageManagedBotCreated" => Some("Managed bot created".to_owned()),
        "messagePaidMessagePriceChanged" => Some("Paid message price changed".to_owned()),
        "messagePaidMessagesRefunded" => Some("Paid messages refunded".to_owned()),
        "messagePassportDataReceived" => Some("Telegram Passport data received".to_owned()),
        "messagePassportDataSent" => Some("Telegram Passport data sent".to_owned()),
        "messagePaymentRefunded" => Some("Payment refunded".to_owned()),
        "messagePaymentSuccessful" | "messagePaymentSuccessfulBot" => {
            Some("Payment successful".to_owned())
        }
        "messagePremiumGiftCode" => Some("Premium gift code".to_owned()),
        "messageProximityAlertTriggered" => Some("Proximity alert".to_owned()),
        "messageRefundedUpgradedGift" => Some("Upgraded gift refunded".to_owned()),
        "messageSuggestBirthdate" => Some("Birthdate suggested".to_owned()),
        "messageSuggestProfilePhoto" => Some("Profile photo suggested".to_owned()),
        "messageSuggestedPostApprovalFailed" => Some("Post approval failed".to_owned()),
        "messageSuggestedPostApproved" => Some("Suggested post approved".to_owned()),
        "messageSuggestedPostDeclined" => Some("Suggested post declined".to_owned()),
        "messageSuggestedPostPaid" => Some("Suggested post paid".to_owned()),
        "messageSuggestedPostRefunded" => Some("Suggested post refunded".to_owned()),
        "messageUpgradedGift" => Some("Upgraded gift".to_owned()),
        "messageUpgradedGiftPurchaseOffer" => Some("Gift purchase offer".to_owned()),
        "messageUpgradedGiftPurchaseOfferRejected" => {
            Some("Gift purchase offer rejected".to_owned())
        }
        "messageUsersShared" => Some("Users shared".to_owned()),
        "messageWebAppDataReceived" => Some("Web app data received".to_owned()),
        "messageWebAppDataSent" => Some("Web app data sent".to_owned()),
        "messageWebsiteConnected" => Some("Website connected".to_owned()),
        "messageUnsupported" => Some("Unsupported Telegram message".to_owned()),
        _ => humanize_message_content_type(content_type),
    }
}

fn nested_formatted_text(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current
        .get("text")
        .and_then(Value::as_str)
        .or_else(|| current.as_str())
        .and_then(|text| (!text.trim().is_empty()).then(|| text.trim().to_owned()))
}

fn trimmed_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn summary_with_detail(label: &str, detail: Option<String>) -> Option<String> {
    Some(match detail {
        Some(detail) => format!("{label}: {detail}"),
        None => label.to_owned(),
    })
}

fn contact_summary(contact: &Value) -> Option<String> {
    let name = [
        trimmed_string(contact.get("first_name")),
        trimmed_string(contact.get("last_name")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
    let phone = trimmed_string(contact.get("phone_number"));
    let detail = match (name.is_empty(), phone) {
        (false, Some(phone)) => format!("{name} · {phone}"),
        (false, None) => name,
        (true, Some(phone)) => phone,
        (true, None) => return Some("Contact".to_owned()),
    };
    Some(format!("Contact: {detail}"))
}

fn location_summary(location: &Value) -> Option<String> {
    let latitude = location.get("latitude").and_then(Value::as_f64)?;
    let longitude = location.get("longitude").and_then(Value::as_f64)?;
    Some(format!("Location: {latitude:.6}, {longitude:.6}"))
}

fn venue_summary(venue: &Value) -> Option<String> {
    let title = trimmed_string(venue.get("title"));
    let address = trimmed_string(venue.get("address"));
    Some(match (title, address) {
        (Some(title), Some(address)) => format!("Venue: {title} · {address}"),
        (Some(title), None) => format!("Venue: {title}"),
        (None, Some(address)) => format!("Venue: {address}"),
        (None, None) => "Venue".to_owned(),
    })
}

fn dice_summary(content: &Value) -> Option<String> {
    let emoji = trimmed_string(content.get("emoji")).unwrap_or_else(|| "Dice".to_owned());
    let value = content
        .get("value")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    Some(if value > 0 {
        format!("{emoji} {value}")
    } else {
        emoji
    })
}

fn humanize_message_content_type(content_type: &str) -> Option<String> {
    let raw = content_type.strip_prefix("message")?;
    if raw.is_empty() {
        return None;
    }
    let mut label = String::with_capacity(raw.len() + 4);
    for (index, character) in raw.chars().enumerate() {
        if index > 0 && character.is_ascii_uppercase() {
            label.push(' ');
        }
        if index > 0 && character.is_ascii_uppercase() {
            label.push(character.to_ascii_lowercase());
        } else {
            label.push(character);
        }
    }
    Some(label)
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum TdlibMessageSenderKey {
    User(String),
    Chat(String),
}

impl TdlibMessageSenderKey {
    fn provider_id(&self) -> &str {
        match self {
            Self::User(value) | Self::Chat(value) => value,
        }
    }

    fn provider_source_key(&self) -> String {
        match self {
            Self::User(value) => format!("user:{value}"),
            Self::Chat(value) => format!("chat:{value}"),
        }
    }

    fn fallback_display_name(&self) -> &'static str {
        match self {
            Self::User(_) => "Telegram user",
            Self::Chat(_) => "Telegram chat",
        }
    }
}

fn message_sender_key(message: &Value) -> Option<TdlibMessageSenderKey> {
    tdlib_sender_key(message.get("sender_id")?)
}

fn participant_sender_key(member: &Value) -> Option<TdlibMessageSenderKey> {
    tdlib_sender_key(member.get("member_id")?)
}

fn tdlib_sender_key(sender: &Value) -> Option<TdlibMessageSenderKey> {
    match sender.get("@type").and_then(Value::as_str) {
        Some("messageSenderUser") => sender
            .get("user_id")
            .and_then(value_id_optional)
            .map(TdlibMessageSenderKey::User),
        Some("messageSenderChat") => sender
            .get("chat_id")
            .and_then(value_id_optional)
            .map(TdlibMessageSenderKey::Chat),
        _ => sender
            .get("user_id")
            .and_then(value_id_optional)
            .map(TdlibMessageSenderKey::User)
            .or_else(|| {
                sender
                    .get("chat_id")
                    .and_then(value_id_optional)
                    .map(TdlibMessageSenderKey::Chat)
            }),
    }
}

fn message_sender_keys(payload: &Value) -> Vec<TdlibMessageSenderKey> {
    // History and search use different TDLib wrapper types (`messages` and
    // `foundMessages`), and older bundled TDLib builds may omit the wrapper
    // discriminator while still returning the typed `messages` field. Resolve
    // senders from the structural contract used by the response parser so no
    // valid message list silently bypasses identity enrichment.
    let mut keys = if let Some(messages) = payload.get("messages").and_then(Value::as_array) {
        messages
            .iter()
            .filter_map(message_sender_key)
            .collect::<Vec<_>>()
    } else if let Some(members) = payload.get("members").and_then(Value::as_array) {
        members
            .iter()
            .filter_map(participant_sender_key)
            .collect::<Vec<_>>()
    } else if let Some(message) = payload.get("message") {
        message_sender_key(message).into_iter().collect()
    } else {
        message_sender_key(payload).into_iter().collect()
    };
    keys.sort();
    keys.dedup();
    keys
}

fn enrich_message_payloads(
    payload: &mut Value,
    known_user_names: &HashMap<String, String>,
    known_chats: &HashMap<String, TelegramChat>,
) {
    if let Some(messages) = payload.get_mut("messages").and_then(Value::as_array_mut) {
        for message in messages {
            enrich_message_sender_name(message, known_user_names, known_chats);
        }
    } else if let Some(members) = payload.get_mut("members").and_then(Value::as_array_mut) {
        for member in members {
            enrich_participant_sender_name(member, known_user_names, known_chats);
        }
    } else if let Some(message) = payload.get_mut("message") {
        enrich_message_sender_name(message, known_user_names, known_chats);
    } else {
        enrich_message_sender_name(payload, known_user_names, known_chats);
    }
}

fn enrich_participant_sender_name(
    member: &mut Value,
    known_user_names: &HashMap<String, String>,
    known_chats: &HashMap<String, TelegramChat>,
) {
    let Some(sender) = participant_sender_key(member) else {
        return;
    };
    let display_name = cached_sender_name(&sender, known_user_names, known_chats);
    if let (Some(display_name), Some(object)) = (display_name, member.as_object_mut()) {
        object.insert("display_name".to_owned(), Value::String(display_name));
    }
}

fn enrich_message_sender_name(
    message: &mut Value,
    known_user_names: &HashMap<String, String>,
    known_chats: &HashMap<String, TelegramChat>,
) {
    let Some(sender) = message_sender_key(message) else {
        return;
    };
    let display_name = cached_sender_name(&sender, known_user_names, known_chats);
    if let (Some(display_name), Some(object)) = (display_name, message.as_object_mut()) {
        object.insert(
            "sender_display_name".to_owned(),
            Value::String(display_name),
        );
    }
}

fn cached_sender_name(
    sender: &TdlibMessageSenderKey,
    known_user_names: &HashMap<String, String>,
    known_chats: &HashMap<String, TelegramChat>,
) -> Option<String> {
    match sender {
        TdlibMessageSenderKey::User(provider_user_id) => {
            known_user_names.get(provider_user_id).cloned()
        }
        TdlibMessageSenderKey::Chat(provider_chat_id) => known_chats
            .get(provider_chat_id)
            .map(|chat| chat.title.clone()),
    }
}

fn tdlib_user_display_name(user: &Value) -> Option<String> {
    let personal_name = [
        bounded_sender_display_name(user.get("first_name")),
        bounded_sender_display_name(user.get("last_name")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
    if !personal_name.is_empty() {
        return bounded_sender_display_name(Some(&Value::String(personal_name)));
    }
    let username = user
        .get("usernames")
        .and_then(|value| value.get("editable_username"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            user.get("usernames")
                .and_then(|value| value.get("active_usernames"))
                .and_then(Value::as_array)
                .and_then(|values| values.first())
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
        })
        .or_else(|| user.get("username").and_then(Value::as_str))?;
    bounded_sender_display_name(Some(&Value::String(format!("@{}", username.trim()))))
}

fn bounded_sender_display_name(value: Option<&Value>) -> Option<String> {
    let value = value?.as_str()?.trim();
    if value.is_empty() || value.chars().any(char::is_control) {
        return None;
    }
    let mut end = value.len().min(256);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    Some(value[..end].to_owned())
}

fn parse_message_media(content: Option<&Value>) -> Option<TelegramMessageMedia> {
    let content = content?;
    let content_type = content.get("@type").and_then(Value::as_str)?;
    let (kind, metadata, file, fallback_content_type, fallback_filename) = match content_type {
        "messagePhoto" => (
            TelegramMediaKind::Photo,
            content
                .get("photo")?
                .get("sizes")?
                .as_array()?
                .last()?
                .get("photo"),
            content
                .get("photo")?
                .get("sizes")?
                .as_array()?
                .last()?
                .get("photo"),
            Some("image/jpeg"),
            Some("photo.jpg".to_owned()),
        ),
        "messageVideo" => (
            TelegramMediaKind::Video,
            content.get("video"),
            content.get("video").and_then(|value| value.get("video")),
            Some("video/mp4"),
            Some("video.mp4".to_owned()),
        ),
        "messageAudio" => (
            TelegramMediaKind::Audio,
            content.get("audio"),
            content.get("audio").and_then(|value| value.get("audio")),
            Some("audio/mpeg"),
            Some("audio.mp3".to_owned()),
        ),
        "messageDocument" => (
            TelegramMediaKind::Document,
            content.get("document"),
            content
                .get("document")
                .and_then(|value| value.get("document")),
            Some("application/octet-stream"),
            Some("document".to_owned()),
        ),
        "messageAnimation" => (
            TelegramMediaKind::Animation,
            content.get("animation"),
            content
                .get("animation")
                .and_then(|value| value.get("animation")),
            Some("video/mp4"),
            Some("animation.mp4".to_owned()),
        ),
        "messageVoiceNote" => (
            TelegramMediaKind::VoiceNote,
            content.get("voice_note"),
            content
                .get("voice_note")
                .and_then(|value| value.get("voice")),
            Some("audio/ogg"),
            Some("voice-note.ogg".to_owned()),
        ),
        "messageVideoNote" => (
            TelegramMediaKind::Video,
            content.get("video_note"),
            content
                .get("video_note")
                .and_then(|value| value.get("video")),
            Some("video/mp4"),
            Some("video-note.mp4".to_owned()),
        ),
        "messageSticker" => {
            let metadata = content.get("sticker");
            let format = metadata
                .and_then(|value| value.get("format"))
                .and_then(|value| value.get("@type"))
                .and_then(Value::as_str);
            let (kind, content_type, extension) = match format {
                Some("stickerFormatTgs") => (
                    TelegramMediaKind::Animation,
                    "application/x-tgsticker",
                    "tgs",
                ),
                Some("stickerFormatWebm") => (TelegramMediaKind::Animation, "video/webm", "webm"),
                _ => (TelegramMediaKind::Photo, "image/webp", "webp"),
            };
            let emoji = metadata
                .and_then(|value| trimmed_string(value.get("emoji")))
                .filter(|value| {
                    value
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '_')
                })
                .unwrap_or_else(|| "sticker".to_owned());
            (
                kind,
                metadata,
                metadata.and_then(|value| value.get("sticker")),
                Some(content_type),
                Some(format!("{emoji}.{extension}")),
            )
        }
        _ => return None,
    };
    let (preview_provider_file_id, preview_content_type) = message_media_preview(content, metadata);
    Some(TelegramMessageMedia {
        kind,
        provider_file_id: file
            .and_then(|value| value.get("id"))
            .and_then(value_id_optional),
        caption: content
            .get("caption")
            .and_then(|value| value.get("text"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        filename: metadata
            .and_then(|value| value.get("file_name"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or(fallback_filename),
        content_type: metadata
            .and_then(|value| value.get("mime_type"))
            .and_then(Value::as_str)
            .filter(valid_content_type)
            .map(ToOwned::to_owned)
            .or_else(|| fallback_content_type.map(ToOwned::to_owned)),
        preview_provider_file_id,
        preview_content_type,
        preview_inline_data: message_media_inline_preview(content, metadata),
        preview_metadata_loaded: true,
    })
}

fn message_media_inline_preview(content: &Value, metadata: Option<&Value>) -> Option<Vec<u8>> {
    let encoded = metadata
        .and_then(|value| value.get("minithumbnail"))
        .and_then(|value| value.get("data"))
        .and_then(Value::as_str)
        .or_else(|| {
            content
                .get("cover")
                .and_then(|value| value.get("minithumbnail"))
                .and_then(|value| value.get("data"))
                .and_then(Value::as_str)
        })?;
    let decoded = STANDARD.decode(encoded).ok()?;
    if decoded.is_empty()
        || decoded.len() > MAX_TDLIB_MINITHUMBNAIL_BYTES
        || !decoded.starts_with(&[0xff, 0xd8])
    {
        return None;
    }
    Some(decoded)
}

fn message_media_preview(
    content: &Value,
    metadata: Option<&Value>,
) -> (Option<String>, Option<String>) {
    let thumbnail_file_id = metadata
        .and_then(|value| value.get("thumbnail"))
        .and_then(|value| value.get("file"))
        .and_then(|value| value.get("id"))
        .and_then(value_id_optional);
    if thumbnail_file_id.is_some() {
        return (
            thumbnail_file_id,
            thumbnail_content_type(metadata).map(ToOwned::to_owned),
        );
    }
    let cover_file_id = content
        .get("cover")
        .and_then(|cover| cover.get("sizes"))
        .and_then(Value::as_array)
        .and_then(|sizes| {
            sizes.iter().find_map(|size| {
                size.get("photo")
                    .and_then(|file| file.get("id"))
                    .and_then(value_id_optional)
            })
        });
    (
        cover_file_id.clone(),
        cover_file_id.map(|_| "image/jpeg".to_owned()),
    )
}

fn thumbnail_content_type(metadata: Option<&Value>) -> Option<&'static str> {
    match metadata?
        .get("thumbnail")?
        .get("format")?
        .get("@type")?
        .as_str()?
    {
        "thumbnailFormatJpeg" => Some("image/jpeg"),
        "thumbnailFormatPng" => Some("image/png"),
        "thumbnailFormatWebp" => Some("image/webp"),
        "thumbnailFormatGif" => Some("image/gif"),
        "thumbnailFormatMpeg4" => Some("video/mp4"),
        _ => None,
    }
}

fn valid_content_type(value: &&str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.is_ascii()
        && value.contains('/')
        && !value.contains(char::is_whitespace)
        && !value.contains(';')
}

fn value_id_optional(value: &Value) -> Option<String> {
    value
        .as_i64()
        .map(|id| id.to_string())
        .or_else(|| value.as_str().map(ToOwned::to_owned))
}

fn optional_trimmed(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn participant_role(status_kind: &str) -> &'static str {
    match status_kind {
        "chatMemberStatusCreator" => "owner",
        "chatMemberStatusAdministrator" => "admin",
        "chatMemberStatusRestricted" => "restricted",
        "chatMemberStatusBanned" => "banned",
        "chatMemberStatusLeft" => "left",
        "chatMemberStatusMember" => "member",
        _ => "unknown",
    }
}

fn participant_permissions(status: Option<&Value>) -> Vec<String> {
    let Some(object) = status.and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut permissions = Vec::new();
    for (key, value) in object {
        if key == "@type" {
            continue;
        }
        if value.as_bool() == Some(true) {
            permissions.push(key.clone());
        } else if let Some(text) = value.as_str().filter(|value| !value.trim().is_empty()) {
            permissions.push(format!("{key}={text}"));
        } else if let Some(nested) = value.as_object() {
            for (nested_key, nested_value) in nested {
                if nested_value.as_bool() == Some(true) {
                    permissions.push(format!("{key}.{nested_key}"));
                }
            }
        }
    }
    permissions.sort();
    permissions
}

fn integer_field(payload: &Value, field: &str) -> Option<u64> {
    payload
        .get(field)
        .and_then(Value::as_i64)
        .and_then(|value| u64::try_from(value).ok())
        .or_else(|| payload.get(field).and_then(Value::as_u64))
}

fn parse_reaction_observations(
    payload: &Value,
) -> Result<Vec<makosh_telegram_api::TelegramReactionObservation>, TdlibError> {
    let Some(values) = payload
        .get("interaction_info")
        .and_then(|value| value.get("reactions"))
        .and_then(|value| value.get("recent_reactions"))
        .and_then(Value::as_array)
    else {
        return Ok(Vec::new());
    };
    values
        .iter()
        .map(|reaction| {
            let sender = reaction
                .get("sender_id")
                .and_then(|sender| sender.get("user_id").or_else(|| sender.get("chat_id")))
                .ok_or_else(|| {
                    TdlibError::Protocol("TDLib reaction sender is missing".to_owned())
                })?;
            let emoji = reaction
                .get("type")
                .and_then(|kind| kind.get("emoji"))
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    TdlibError::Protocol("TDLib reaction emoji is missing".to_owned())
                })?;
            Ok(makosh_telegram_api::TelegramReactionObservation {
                sender_id: value_id(sender)?,
                emoji: emoji.to_owned(),
                is_outgoing: reaction
                    .get("is_outgoing")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_active: true,
            })
        })
        .collect()
}

fn value_id(value: &Value) -> Result<String, TdlibError> {
    value
        .as_i64()
        .map(|id| id.to_string())
        .or_else(|| value.as_str().map(ToOwned::to_owned))
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| TdlibError::Protocol("TDLib provider id is invalid".to_owned()))
}

fn required_string(payload: &Value, field: &str) -> Result<String, TdlibError> {
    payload
        .get(field)
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
        .or_else(|| {
            payload
                .get(field)
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| TdlibError::Protocol(format!("TDLib payload field `{field}` is missing")))
}

fn chat_kind(value: &str) -> Result<TelegramChatKind, TdlibError> {
    match value {
        "chatTypePrivate" => Ok(TelegramChatKind::Private),
        "chatTypeBasicGroup" | "chatTypeSupergroup" => Ok(TelegramChatKind::Group),
        "chatTypeSecret" => Ok(TelegramChatKind::Private),
        other => Err(TdlibError::Protocol(format!(
            "unsupported TDLib chat type `{other}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_pending_update_draining_without_dropping_payloads() {
        let mut pending_updates = (0..20)
            .map(|sequence| json!({"sequence": sequence}))
            .collect::<VecDeque<_>>();

        let drained =
            drain_pending_update_payloads(&mut pending_updates, MAX_TDLIB_UPDATE_PAYLOADS_PER_POLL);

        assert_eq!(drained.len(), MAX_TDLIB_UPDATE_PAYLOADS_PER_POLL);
        assert_eq!(
            drained.first().and_then(|value| value["sequence"].as_i64()),
            Some(0)
        );
        assert_eq!(
            drained.last().and_then(|value| value["sequence"].as_i64()),
            Some(15)
        );
        assert_eq!(pending_updates.len(), 4);
        assert_eq!(
            pending_updates
                .front()
                .and_then(|value| value["sequence"].as_i64()),
            Some(16)
        );
    }

    #[test]
    fn authorization_parameters_debug_redacts_credentials_and_session_path() {
        let parameters = TdlibAuthorizationParameters {
            api_id: 42,
            api_hash: Zeroizing::new("private-api-hash".to_owned()),
            database_directory: PathBuf::from("/private/provider-session"),
            session_encryption_key: Some(Zeroizing::new(b"private-session-key".to_vec())),
        };

        let diagnostic = format!("{parameters:?}");

        assert!(diagnostic.contains("[redacted]"));
        assert!(!diagnostic.contains("42"));
        assert!(!diagnostic.contains("private-api-hash"));
        assert!(!diagnostic.contains("/private/provider-session"));
        assert!(!diagnostic.contains("private-session-key"));
    }

    #[test]
    fn provider_error_translation_drops_untrusted_message_content() {
        let error = tdlib_error(&json!({
            "code": 401,
            "message": "private-message-and-credential-sentinel"
        }));

        let diagnostic = format!("{error:?}");

        assert!(diagnostic.contains("401"));
        assert!(!diagnostic.contains("private-message-and-credential-sentinel"));
    }

    #[test]
    fn authorization_event_debug_redacts_qr_links_hints_and_provider_messages() {
        let password = parse_authorization_update(&json!({
            "@type": "authorizationStateWaitPassword",
            "password_hint": "private-password-hint"
        }))
        .expect("password authorization update");
        let provider_error = parse_authorization_update(&json!({
            "@type": "error",
            "code": 401,
            "message": "private-provider-error"
        }))
        .expect("provider authorization error");
        let qr = TdlibAuthorizationEvent::QrLink("tg://private-qr-token".to_owned());

        let diagnostic = format!("{password:?} {provider_error:?} {qr:?}");

        assert!(diagnostic.contains("[redacted]"));
        assert!(!diagnostic.contains("private-password-hint"));
        assert!(!diagnostic.contains("private-provider-error"));
        assert!(!diagnostic.contains("tg://private-qr-token"));
    }

    #[test]
    fn nested_authorization_state_preserves_password_hint_and_qr_link() {
        let password = parse_authorization_update(&json!({
            "@type": "updateAuthorizationState",
            "authorization_state": {
                "@type": "authorizationStateWaitPassword",
                "password_hint": "private-password-hint"
            }
        }))
        .expect("nested password authorization update");
        let qr_link = parse_qr_authorization_link(&json!({
            "@type": "updateAuthorizationState",
            "authorization_state": {
                "@type": "authorizationStateWaitOtherDeviceConfirmation",
                "link": "tg://private-qr-token"
            }
        }))
        .expect("nested QR authorization link");

        assert!(matches!(
            password,
            TdlibAuthorizationUpdate::WaitingPassword { hint: Some(_) }
        ));
        assert_eq!(qr_link, "tg://private-qr-token");
    }

    #[test]
    fn tdlib_logging_is_disabled_before_provider_configuration() {
        assert_eq!(
            disable_tdlib_logging_request(),
            json!({
                "@type": "setLogVerbosityLevel",
                "new_verbosity_level": 0,
            }),
        );
    }

    #[test]
    fn edit_command_encodes_tdlib_message_operation_without_domain_fields() {
        let command = TelegramProviderCommand::Edit {
            operation_id: "op-edit".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            provider_message_id: "200".to_owned(),
            text: "updated".to_owned(),
        };
        let encoded = encode_provider_command(&command).expect("valid edit command");
        assert_eq!(encoded["@type"], "editMessageText");
        assert_eq!(encoded["chat_id"], 100);
        assert_eq!(encoded["message_id"], 200);
        assert_eq!(encoded["@extra"], "op-edit");
    }

    #[test]
    fn mark_read_uses_view_messages_when_cursor_is_present() {
        let command = TelegramProviderCommand::MarkUnread {
            operation_id: "op-read".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            unread: false,
            read_through_provider_message_id: Some("200".to_owned()),
        };
        let encoded = encode_provider_command(&command).expect("valid read command");
        assert_eq!(encoded["@type"], "viewMessages");
        assert_eq!(encoded["message_ids"][0], 200);
    }

    #[test]
    fn parses_content_update_into_message_edit_event() {
        let events = parse_provider_events(
            "account",
            &json!({
                "@type": "updateMessageContent",
                "chat_id": 100,
                "message_id": 200,
                "new_content": {
                    "@type": "messageText",
                    "text": {"@type": "formattedText", "text": "edited"}
                }
            }),
        )
        .expect("content update");
        assert!(matches!(
            &events[0],
            TelegramProviderEvent::MessageEdited { text: Some(text), .. } if text == "edited"
        ));
    }

    #[test]
    fn parses_delete_update_as_one_event_per_provider_message() {
        let events = parse_provider_events(
            "account",
            &json!({
                "@type": "updateDeleteMessages",
                "chat_id": 100,
                "message_ids": [200, 201],
                "is_permanent": true
            }),
        )
        .expect("delete update");
        assert_eq!(events.len(), 2);
        assert!(matches!(
            &events[0],
            TelegramProviderEvent::MessageDeleted { provider_message_id, is_permanent: true, .. }
                if provider_message_id == "200"
        ));
    }

    #[test]
    fn encodes_media_download_and_participant_requests_at_provider_boundary() {
        let media = TelegramProviderCommand::SendMedia(makosh_telegram_api::TelegramSendMedia {
            operation_id: "op-media".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            media_kind: TelegramMediaKind::Document,
            blob: makosh_telegram_api::TelegramBlobIntentV1 {
                blob_ref: "blob:report".to_owned(),
                reference_id: vec![7; 32],
                declared_size: 42,
                backup_class: 1,
            },
            caption: Some("report".to_owned()),
            filename: Some("report.pdf".to_owned()),
        });
        let encoded = encode_send_media_materialized(
            match &media {
                TelegramProviderCommand::SendMedia(command) => command,
                _ => unreachable!(),
            },
            "/tmp/report.pdf",
        )
        .expect("valid materialized media command");
        assert_eq!(
            encoded["input_message_content"]["@type"],
            "inputMessageDocument"
        );
        assert_eq!(
            encoded["input_message_content"]["document"]["path"],
            "/tmp/report.pdf"
        );
        assert!(encode_provider_command(&media).is_err());

        let participants = TelegramProviderCommand::ListParticipants {
            operation_id: "op-members".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            filter: TelegramParticipantFilter::Administrators,
            offset: 10,
            limit: 50,
        };
        let encoded = encode_provider_command(&participants).expect("valid participant command");
        assert_eq!(encoded["@type"], "getSupergroupMembers");
        assert_eq!(
            encoded["filter"]["@type"],
            "supergroupMembersFilterAdministrators"
        );
        assert_eq!(encoded["@extra"], "op-members");

        let recent_participants = TelegramProviderCommand::ListParticipants {
            operation_id: "op-recent-members".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            filter: TelegramParticipantFilter::Recent,
            offset: 0,
            limit: 50,
        };
        let encoded = encode_provider_command(&recent_participants)
            .expect("valid recent participant command");
        assert_eq!(encoded["filter"]["@type"], "supergroupMembersFilterRecent");
    }

    #[test]
    fn parses_provider_file_snapshot_without_exposing_raw_tdlib_payload() {
        let file = parse_file_snapshot(
            "account",
            &json!({
                "@type": "file",
                "id": 42,
                "size": 100,
                "expected_size": 100,
                "local": {"path": "/tmp/file", "downloaded_size": 100, "is_downloading_completed": true},
                "remote": {"unique_id": "remote-42"}
            }),
        )
        .expect("valid file snapshot");
        assert_eq!(file.provider_file_id, "42");
        assert_eq!(file.provider_unique_id.as_deref(), Some("remote-42"));
        assert!(file.is_downloaded);
    }

    #[test]
    fn captures_completed_download_response_as_private_provider_update() {
        let completed = json!({
            "@type": "file",
            "id": 42,
            "size": 100,
            "local": {
                "path": "/private/provider/file",
                "downloaded_size": 100,
                "is_downloading_completed": true
            },
            "remote": {"unique_id": "remote-42"}
        });
        let pending = json!({
            "@type": "file",
            "id": 43,
            "size": 100,
            "local": {
                "path": "",
                "downloaded_size": 0,
                "is_downloading_completed": false
            }
        });

        let update = completed_download_response_update("account", &completed)
            .expect("completed download response")
            .expect("completed file must become an update");
        assert_eq!(update["@type"], "updateFile");
        assert!(
            completed_download_response_update("account", &pending)
                .expect("pending download response")
                .is_none()
        );

        let command = TelegramDownloadFile {
            operation_id: "operation".to_owned(),
            account_id: "account".to_owned(),
            provider_file_id: "42".to_owned(),
            priority: 1,
        };
        assert!(is_download_file_request(&TdlibRequest::DownloadFile(
            command.clone()
        )));
        assert!(is_download_file_request(&TdlibRequest::ProviderCommand(
            TelegramProviderCommand::DownloadFile(command)
        )));
    }

    #[test]
    fn parses_provider_participant_page_with_provider_roles() {
        let page = parse_participant_page(
            "account",
            "100",
            TelegramParticipantFilter::Recent,
            0,
            &json!({
                "members": [
                    {"member_id": {"user_id": 7}, "status": {"@type": "chatMemberStatusCreator"}},
                    {"member_id": {"user_id": 8}, "status": {"@type": "chatMemberStatusMember"}}
                ]
            }),
        )
        .expect("valid participant page");
        assert_eq!(page.items.len(), 2);
        assert!(page.items[0].is_owner);
        assert!(!page.items[1].is_admin);
        assert_eq!(page.next_offset, Some(2));
    }

    #[test]
    fn parses_message_media_file_and_caption_without_business_mapping() {
        let message = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 200,
                "date": 10,
                "is_outgoing": false,
                "sender_id": {"user_id": 7},
                "content": {
                    "@type": "messageDocument",
                    "document": {"file_name": "report.pdf", "document": {"id": 42}},
                    "caption": {"text": "report"}
                }
            }),
        )
        .expect("document message");
        let media = message.media.expect("media snapshot");
        assert_eq!(media.kind, TelegramMediaKind::Document);
        assert_eq!(media.provider_file_id.as_deref(), Some("42"));
        assert_eq!(media.caption.as_deref(), Some("report"));
        assert_eq!(media.filename.as_deref(), Some("report.pdf"));
        assert_eq!(message.text.as_deref(), Some("report"));
    }

    #[test]
    fn every_current_tdlib_message_content_type_has_a_renderable_summary() {
        for content_type in CURRENT_TDLIB_MESSAGE_CONTENT_TYPES {
            let content = json!({"@type": content_type});
            let summary = message_text(Some(&content));
            assert!(
                summary
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty()),
                "missing summary for {content_type}",
            );
            assert!(
                !summary
                    .as_deref()
                    .is_some_and(|value| value.starts_with('[')),
                "placeholder summary for {content_type}",
            );
        }
    }

    #[test]
    fn parsed_messages_use_the_resolved_tdlib_sender_name() {
        let message = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 200,
                "date": 10,
                "is_outgoing": false,
                "sender_id": {"@type": "messageSenderUser", "user_id": 7},
                "sender_display_name": "Ada Lovelace",
                "content": {"@type": "messageText", "text": {"text": "hello"}}
            }),
        )
        .expect("message");

        assert_eq!(message.sender_id, "7");
        assert_eq!(message.sender_display_name.as_deref(), Some("Ada Lovelace"));
    }

    #[test]
    fn anonymous_chat_messages_prefer_the_provider_author_signature() {
        let message = parse_message_observation(
            "account",
            &json!({
                "chat_id": -1007,
                "id": 200,
                "date": 10,
                "is_outgoing": false,
                "sender_id": {"@type": "messageSenderChat", "chat_id": -1007},
                "sender_display_name": "Channel title",
                "author_signature": "Public author",
                "content": {"@type": "messageText", "text": {"text": "hello"}}
            }),
        )
        .expect("anonymous chat message");

        assert_eq!(
            message.sender_display_name.as_deref(),
            Some("Public author")
        );
    }

    #[test]
    fn parsed_messages_never_expose_an_opaque_sender_when_tdlib_has_no_name() {
        let user_message = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 200,
                "date": 10,
                "is_outgoing": false,
                "sender_id": {"@type": "messageSenderUser", "user_id": 7},
                "content": {"@type": "messageText", "text": {"text": "hello"}}
            }),
        )
        .expect("user message");
        let chat_message = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 201,
                "date": 10,
                "is_outgoing": false,
                "sender_id": {"@type": "messageSenderChat", "chat_id": -1007},
                "content": {"@type": "messageText", "text": {"text": "hello"}}
            }),
        )
        .expect("chat message");

        assert_eq!(
            user_message.sender_display_name.as_deref(),
            Some("Telegram user")
        );
        assert_eq!(
            chat_message.sender_display_name.as_deref(),
            Some("Telegram chat")
        );
    }

    #[test]
    fn tdlib_user_name_prefers_personal_name_and_falls_back_to_username() {
        assert_eq!(
            tdlib_user_display_name(&json!({
                "first_name": "Ada",
                "last_name": "Lovelace",
                "usernames": {"active_usernames": ["ada"]}
            }))
            .as_deref(),
            Some("Ada Lovelace"),
        );
        assert_eq!(
            tdlib_user_display_name(&json!({
                "first_name": "",
                "last_name": "",
                "usernames": {"active_usernames": ["ada"]}
            }))
            .as_deref(),
            Some("@ada"),
        );
    }

    #[test]
    fn enriches_sender_names_for_history_and_search_message_wrappers() {
        let known_user_names = HashMap::from([("7".to_owned(), "Ada Lovelace".to_owned())]);
        let known_chats = HashMap::new();
        for wrapper_type in ["messages", "foundMessages"] {
            let mut response = json!({
                "@type": wrapper_type,
                "messages": [{
                    "@type": "message",
                    "chat_id": 100,
                    "id": 200,
                    "date": 10,
                    "is_outgoing": false,
                    "sender_id": {"@type": "messageSenderUser", "user_id": 7},
                    "content": {"@type": "messageText", "text": {"text": "hello"}}
                }]
            });

            assert_eq!(
                message_sender_keys(&response),
                vec![TdlibMessageSenderKey::User("7".to_owned())],
            );
            enrich_message_payloads(&mut response, &known_user_names, &known_chats);
            assert_eq!(
                response["messages"][0]["sender_display_name"],
                "Ada Lovelace"
            );
        }
    }

    #[test]
    fn enriches_sender_names_when_an_older_tdlib_wrapper_has_no_discriminator() {
        let known_user_names = HashMap::from([("7".to_owned(), "@ada".to_owned())]);
        let known_chats = HashMap::new();
        let mut response = json!({
            "messages": [{
                "@type": "message",
                "chat_id": 100,
                "id": 200,
                "date": 10,
                "is_outgoing": false,
                "sender_id": {"@type": "messageSenderUser", "user_id": 7},
                "content": {"@type": "messageText", "text": {"text": "hello"}}
            }]
        });

        enrich_message_payloads(&mut response, &known_user_names, &known_chats);

        assert_eq!(response["messages"][0]["sender_display_name"], "@ada");
    }

    #[test]
    fn enriches_participant_names_from_the_same_tdlib_user_directory() {
        let known_user_names = HashMap::from([("7".to_owned(), "Ada Lovelace".to_owned())]);
        let known_chats = HashMap::new();
        let mut response = json!({
            "@type": "chatMembers",
            "total_count": 1,
            "members": [{
                "@type": "chatMember",
                "member_id": {"@type": "messageSenderUser", "user_id": 7},
                "status": {"@type": "chatMemberStatusMember"}
            }]
        });

        assert_eq!(
            message_sender_keys(&response),
            vec![TdlibMessageSenderKey::User("7".to_owned())],
        );
        enrich_message_payloads(&mut response, &known_user_names, &known_chats);
        let page = parse_participant_page(
            "account",
            "100",
            TelegramParticipantFilter::Recent,
            0,
            &response,
        )
        .expect("participant page");

        assert_eq!(page.items[0].display_name.as_deref(), Some("Ada Lovelace"));
    }

    #[test]
    fn parses_video_thumbnail_separately_from_the_full_provider_file() {
        let message = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 200,
                "date": 10,
                "is_outgoing": false,
                "sender_id": {"user_id": 7},
                "content": {
                    "@type": "messageVideo",
                    "video": {
                        "mime_type": "video/mp4",
                        "minithumbnail": {
                            "width": 8,
                            "height": 8,
                            "data": STANDARD.encode([0xff, 0xd8, 0xff, 0xd9])
                        },
                        "thumbnail": {
                            "format": {"@type": "thumbnailFormatJpeg"},
                            "file": {"id": 41}
                        },
                        "video": {"id": 42}
                    },
                    "caption": {"text": "video"}
                }
            }),
        )
        .expect("video message");
        let media = message.media.expect("media snapshot");
        assert_eq!(media.kind, TelegramMediaKind::Video);
        assert_eq!(media.provider_file_id.as_deref(), Some("42"));
        assert_eq!(media.preview_provider_file_id.as_deref(), Some("41"));
        assert_eq!(media.preview_content_type.as_deref(), Some("image/jpeg"));
        assert_eq!(
            media.preview_inline_data,
            Some(vec![0xff, 0xd8, 0xff, 0xd9])
        );
        assert!(media.preview_metadata_loaded);
    }

    #[test]
    fn uses_the_smallest_video_cover_when_the_provider_thumbnail_is_absent() {
        let message = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 200,
                "date": 10,
                "is_outgoing": false,
                "sender_id": {"user_id": 7},
                "content": {
                    "@type": "messageVideo",
                    "video": {
                        "mime_type": "video/mp4",
                        "thumbnail": null,
                        "video": {"id": 42}
                    },
                    "cover": {
                        "minithumbnail": {
                            "width": 8,
                            "height": 8,
                            "data": STANDARD.encode([0xff, 0xd8, 0xff, 0xd9])
                        },
                        "sizes": [
                            {"type": "s", "photo": {"id": 40}},
                            {"type": "x", "photo": {"id": 41}}
                        ]
                    },
                    "caption": {"text": ""}
                }
            }),
        )
        .expect("covered video message");
        let media = message.media.expect("media snapshot");

        assert_eq!(media.provider_file_id.as_deref(), Some("42"));
        assert_eq!(media.preview_provider_file_id.as_deref(), Some("40"));
        assert_eq!(media.preview_content_type.as_deref(), Some("image/jpeg"));
        assert_eq!(
            media.preview_inline_data,
            Some(vec![0xff, 0xd8, 0xff, 0xd9])
        );
        assert!(media.preview_metadata_loaded);
    }

    #[test]
    fn parses_sticker_and_video_note_as_downloadable_provider_media() {
        let sticker = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 200,
                "date": 10,
                "is_outgoing": false,
                "sender_id": {"user_id": 7},
                "content": {
                    "@type": "messageSticker",
                    "sticker": {
                        "emoji": "ok",
                        "format": {"@type": "stickerFormatWebp"},
                        "thumbnail": {
                            "format": {"@type": "thumbnailFormatWebp"},
                            "file": {"id": 41}
                        },
                        "sticker": {"id": 42}
                    }
                }
            }),
        )
        .expect("sticker message");
        let sticker_media = sticker.media.expect("sticker media");
        assert_eq!(sticker.text.as_deref(), Some("ok"));
        assert_eq!(sticker_media.kind, TelegramMediaKind::Photo);
        assert_eq!(sticker_media.provider_file_id.as_deref(), Some("42"));
        assert_eq!(
            sticker_media.preview_provider_file_id.as_deref(),
            Some("41")
        );
        assert_eq!(sticker_media.content_type.as_deref(), Some("image/webp"));
        assert_eq!(sticker_media.filename.as_deref(), Some("ok.webp"));

        let video_note = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 201,
                "date": 11,
                "is_outgoing": false,
                "sender_id": {"user_id": 7},
                "content": {
                    "@type": "messageVideoNote",
                    "video_note": {
                        "thumbnail": {
                            "format": {"@type": "thumbnailFormatJpeg"},
                            "file": {"id": 51}
                        },
                        "video": {"id": 52}
                    }
                }
            }),
        )
        .expect("video note message");
        let video_note_media = video_note.media.expect("video note media");
        assert_eq!(video_note.text.as_deref(), Some("Video message"));
        assert_eq!(video_note_media.kind, TelegramMediaKind::Video);
        assert_eq!(video_note_media.provider_file_id.as_deref(), Some("52"));
        assert_eq!(
            video_note_media.preview_provider_file_id.as_deref(),
            Some("51")
        );
        assert_eq!(video_note_media.content_type.as_deref(), Some("video/mp4"));
    }

    #[test]
    fn preserves_structured_and_service_message_meaning_in_display_text() {
        let cases = [
            (
                json!({
                    "@type": "messagePoll",
                    "poll": {"question": {"text": "Choose one"}}
                }),
                "Poll: Choose one",
            ),
            (
                json!({
                    "@type": "messageContact",
                    "contact": {
                        "first_name": "Ada",
                        "last_name": "Lovelace",
                        "phone_number": "+34000000000"
                    }
                }),
                "Contact: Ada Lovelace · +34000000000",
            ),
            (
                json!({
                    "@type": "messageVenue",
                    "venue": {"title": "Cafe", "address": "Main street"}
                }),
                "Venue: Cafe · Main street",
            ),
            (
                json!({"@type": "messageDice", "emoji": "🎲", "value": 6}),
                "🎲 6",
            ),
            (
                json!({"@type": "messageChatChangeTitle", "title": "Home"}),
                "Chat title changed: Home",
            ),
            (
                json!({"@type": "messageForumTopicCreated"}),
                "Forum topic created",
            ),
        ];

        for (index, (content, expected)) in cases.into_iter().enumerate() {
            let message = parse_message_observation(
                "account",
                &json!({
                    "chat_id": 100,
                    "id": 300 + index,
                    "date": 10,
                    "is_outgoing": false,
                    "sender_id": {"user_id": 7},
                    "content": content
                }),
            )
            .expect("structured message");
            assert_eq!(message.text.as_deref(), Some(expected));
        }
    }

    #[test]
    fn preserves_media_meaning_when_the_provider_caption_is_empty() {
        let message = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 200,
                "date": 10,
                "is_outgoing": false,
                "sender_id": {"user_id": 7},
                "content": {
                    "@type": "messageDocument",
                    "document": {"file_name": "report.pdf", "document": {"id": 42}},
                    "caption": {"text": "   "}
                }
            }),
        )
        .expect("document message with no caption");

        assert_eq!(message.text.as_deref(), Some("File"));
    }

    #[test]
    fn parses_typing_and_topic_updates_as_provider_events() {
        let typing = parse_provider_events(
            "account",
            &json!({
                "@type": "updateUserChatAction",
                "chat_id": 100,
                "message_thread_id": 7,
                "sender_id": {"@type": "messageSenderUser", "user_id": 42},
                "action": {"@type": "chatActionTyping"}
            }),
        )
        .expect("typing update");
        assert!(matches!(
            &typing[0],
            TelegramProviderEvent::TypingChanged(state)
                if state.sender_id == "user:42" && state.provider_thread_id.as_deref() == Some("7")
        ));

        let topic = parse_provider_events(
            "account",
            &json!({
                "@type": "updateForumTopicInfo",
                "chat_id": 100,
                "info": {"message_thread_id": 7, "name": "Release", "is_pinned": true, "is_closed": false}
            }),
        )
        .expect("topic update");
        assert!(matches!(
            &topic[0],
            TelegramProviderEvent::TopicChanged(topic)
                if topic.provider_topic_id == "7" && topic.title == "Release" && topic.is_pinned
        ));
    }

    #[test]
    fn parses_provider_chat_position_folder_and_notification_updates() {
        let position = parse_provider_events(
            "account",
            &json!({
                "@type": "updateChatPosition",
                "chat_id": 100,
                "position": {
                    "list": {"@type": "chatListFolder", "chat_folder_id": 7},
                    "order": 9,
                    "is_pinned": true
                }
            }),
        )
        .expect("chat position update");
        assert!(matches!(
            &position[0],
            TelegramProviderEvent::ChatPositionChanged(position)
                if position.list_kind == "folder" && position.provider_folder_id == Some(7) && position.is_pinned
        ));

        let folders = parse_provider_events(
            "account",
            &json!({
                "@type": "updateChatFolders",
                "chat_folders": [{"id": 7, "name": {"text": "Projects"}, "icon": {"name": "briefcase"}, "color_id": 3}]
            }),
        )
        .expect("chat folders update");
        assert!(matches!(
            &folders[0],
            TelegramProviderEvent::ChatFoldersChanged { folders, .. }
                if folders[0].title == "Projects" && folders[0].provider_folder_id == 7
        ));

        let notification = parse_provider_events(
            "account",
            &json!({
                "@type": "updateChatNotificationSettings",
                "chat_id": 100,
                "notification_settings": {"use_default_mute_for": false, "mute_for": 3600}
            }),
        )
        .expect("notification update");
        assert!(matches!(
            &notification[0],
            TelegramProviderEvent::ChatNotificationChanged {
                mute_for_seconds: 3600,
                ..
            }
        ));
    }

    #[test]
    fn ignores_provider_chat_positions_from_unprojected_lists() {
        let events = parse_provider_events(
            "account",
            &json!({
                "@type": "updateChatPosition",
                "chat_id": 100,
                "position": {
                    "list": {"@type": "chatListFilter"},
                    "order": 9,
                    "is_pinned": false
                }
            }),
        )
        .expect("unsupported list is a non-fatal provider update");

        assert!(events.is_empty());
    }

    #[test]
    fn accepts_provider_interaction_info_without_recent_reactions() {
        let events = parse_provider_events(
            "account",
            &json!({
                "@type": "updateMessageInteractionInfo",
                "chat_id": 100,
                "message_id": 42,
                "interaction_info": {
                    "@type": "messageInteractionInfo",
                    "view_count": 1,
                    "forward_count": 0,
                    "reactions": {
                        "@type": "messageReactions",
                        "reactions": []
                    }
                }
            }),
        )
        .expect("missing recent reactions are a non-fatal provider update");

        assert!(matches!(
            events.as_slice(),
            [TelegramProviderEvent::ReactionsObserved { reactions, .. }] if reactions.is_empty()
        ));
    }

    #[test]
    fn loads_provider_chat_catalog_before_reading_cached_chat_ids() {
        let request = encode_load_chats_page_request(100, "load-account-1");
        assert_eq!(request["@type"], "loadChats");
        assert!(request["chat_list"].is_null());
        assert_eq!(request["limit"], 100);
        assert_eq!(request["@extra"], "load-account-1");

        let list_request = encode_request(&TdlibRequest::LoadChats {
            account_id: "account-1".to_owned(),
            limit: 100,
        })
        .expect("chat catalog query");
        assert_eq!(list_request["@type"], "getChats");
        assert!(list_request["chat_list"].is_null());

        validate_load_chats_page_response(&json!({"@type": "ok"}))
            .expect("loaded provider chat page");
        validate_load_chats_page_response(&json!({"@type": "error", "code": 404}))
            .expect("provider reports that the full chat catalog is already loaded");
        assert!(load_chats_page_exhausted(
            &json!({"@type": "error", "code": 404})
        ));
        assert!(!load_chats_page_exhausted(&json!({"@type": "ok"})));
        assert_eq!(
            validate_load_chats_page_response(&json!({
                "@type": "error",
                "code": 400,
                "message": "private provider detail"
            })),
            Err(TdlibError::Protocol("TDLib error 400".to_owned())),
        );
        assert!(should_load_next_chat_page(0, 100, 5_000, false));
        assert!(!should_load_next_chat_page(100, 100, 5_000, false));
        assert!(!should_load_next_chat_page(100, 200, 5_000, true));
        assert!(!should_load_next_chat_page(100, 200, 200, false));
    }

    #[test]
    fn retains_new_chat_updates_for_catalogs_larger_than_get_chats() {
        let chat = parse_new_chat_update(
            "account-1",
            &json!({
                "@type": "updateNewChat",
                "chat": {
                    "@type": "chat",
                    "id": -1001234567890_i64,
                    "title": "Archive",
                    "type": {"@type": "chatTypeSupergroup"},
                    "photo": {
                        "small": {"id": 77, "remote": {"unique_id": "avatar-77"}}
                    }
                }
            }),
        )
        .expect("valid new chat update")
        .expect("chat update");

        assert_eq!(chat.provider_chat_id, "-1001234567890");
        assert_eq!(chat.title, "Archive");
        assert_eq!(chat.kind, TelegramChatKind::Group);
        assert_eq!(chat.avatar_provider_file_id.as_deref(), Some("77"));
        assert_eq!(chat.avatar_provider_unique_id.as_deref(), Some("avatar-77"));
        assert_eq!(
            parse_new_chat_update("account-1", &json!({"@type": "ok"})),
            Ok(None)
        );
    }

    #[test]
    fn retains_plain_get_chat_responses_for_message_sender_chat_labels() {
        let chat = parse_new_chat_update(
            "account-1",
            &json!({
                "@type": "chat",
                "id": -1001234567890_i64,
                "title": "Channel name",
                "type": {"@type": "chatTypeSupergroup"},
            }),
        )
        .expect("valid getChat response")
        .expect("chat snapshot");

        assert_eq!(chat.provider_chat_id, "-1001234567890");
        assert_eq!(chat.title, "Channel name");
        assert_eq!(chat.kind, TelegramChatKind::Group);
    }

    #[test]
    fn derives_private_sender_identity_from_chat_catalog_snapshots() {
        let update = json!({
            "@type": "updateNewChat",
            "chat": {
                "@type": "chat",
                "id": 42,
                "title": "Ada Lovelace",
                "type": {"@type": "chatTypePrivate", "user_id": 7}
            }
        });
        let snapshot = update["chat"].clone();

        assert_eq!(private_chat_provider_user_id(&update).as_deref(), Some("7"));
        assert_eq!(
            private_chat_provider_user_id(&snapshot).as_deref(),
            Some("7")
        );
        assert_eq!(
            private_chat_provider_user_id(&json!({
                "@type": "chat",
                "id": -1007,
                "title": "Group",
                "type": {"@type": "chatTypeSupergroup", "supergroup_id": 7}
            })),
            None,
        );
    }

    #[test]
    fn ignores_partial_new_chat_updates_without_a_title() {
        assert_eq!(
            parse_new_chat_update(
                "account-1",
                &json!({
                    "@type": "updateNewChat",
                    "chat": {
                        "@type": "chat",
                        "id": -1001234567890_i64,
                        "type": {"@type": "chatTypeSupergroup"}
                    }
                }),
            ),
            Ok(None),
        );
    }

    #[test]
    fn labels_own_private_chat_as_saved_messages() {
        let mut chats = vec![
            TelegramChat {
                account_id: "account-1".to_owned(),
                provider_chat_id: "42".to_owned(),
                kind: TelegramChatKind::Private,
                title: "Owner display name".to_owned(),
                username: None,
                avatar_provider_file_id: None,
                avatar_provider_unique_id: None,
            },
            TelegramChat {
                account_id: "account-1".to_owned(),
                provider_chat_id: "84".to_owned(),
                kind: TelegramChatKind::Private,
                title: "Contact".to_owned(),
                username: None,
                avatar_provider_file_id: None,
                avatar_provider_unique_id: None,
            },
        ];

        label_saved_messages_chat(&mut chats, "42");

        assert_eq!(chats[0].title, "Saved Messages");
        assert_eq!(chats[1].title, "Contact");
    }

    #[test]
    fn preserves_signed_tdlib_chat_ids_without_weakening_other_provider_ids() {
        assert_eq!(signed_chat_id("-1001234567890"), Ok(-1_001_234_567_890));
        assert_eq!(signed_chat_id("42"), Ok(42));
        assert!(signed_chat_id("0").is_err());
        assert!(provider_id("-1001234567890").is_err());

        let request = encode_request(&TdlibRequest::LoadHistory {
            account_id: "account-1".to_owned(),
            provider_chat_id: "-1001234567890".to_owned(),
            from_message_id: None,
            mode: makosh_telegram_api::TelegramHistorySyncMode::Latest,
            limit: 100,
        })
        .expect("signed chat history request");
        assert_eq!(request["chat_id"], -1_001_234_567_890_i64);
    }

    #[test]
    fn encodes_exact_message_repair_request_for_signed_chat_ids() {
        let request = get_message_request("account-1", "-1001234567890", "42")
            .and_then(|request| encode_request(&request))
            .expect("exact message request");

        assert_eq!(request["@type"], "getMessage");
        assert_eq!(request["chat_id"], -1_001_234_567_890_i64);
        assert_eq!(request["message_id"], 42);
        assert_eq!(request["@extra"], "account-1:message:42");
        assert!(get_message_request("account-1", "-1001234567890", "0").is_err());
    }

    #[test]
    fn resolves_observed_message_senders_by_exact_tdlib_identity_kind() {
        let user_request = TdlibRequest::ResolveSender {
            correlation_id: "resolve-user".to_owned(),
            provider_sender_id: "42".to_owned(),
        };
        let chat_request = TdlibRequest::ResolveSender {
            correlation_id: "resolve-chat".to_owned(),
            provider_sender_id: "-100700".to_owned(),
        };

        let encoded_user = encode_request(&user_request).expect("user sender lookup");
        assert_eq!(encoded_user["@type"], "getUser");
        assert_eq!(encoded_user["user_id"], 42);
        assert_eq!(encoded_user["@extra"], "resolve-user");

        let encoded_chat = encode_request(&chat_request).expect("chat sender lookup");
        assert_eq!(encoded_chat["@type"], "getChat");
        assert_eq!(encoded_chat["chat_id"], -100700_i64);
        assert_eq!(encoded_chat["@extra"], "resolve-chat");

        let response = parse_response_for_request(
            "account",
            &user_request,
            json!({
                "@type": "user",
                "id": 42,
                "first_name": "Visible",
                "last_name": "Name",
                "usernames": {"active_usernames": ["visible_name"]}
            }),
        )
        .expect("resolved user name");
        assert_eq!(
            response,
            TdlibResponse::SenderName {
                provider_sender_id: "42".to_owned(),
                display_name: Some("Visible Name".to_owned()),
            }
        );
    }

    #[test]
    fn preserves_tdlib_file_progress_as_provider_event() {
        let events = parse_provider_events(
            "account",
            &json!({
                "@type": "updateFile",
                "file": {
                    "@type": "file",
                    "id": 42,
                    "size": 100,
                    "local": {"downloaded_size": 40, "is_downloading_active": true, "is_downloading_completed": false}
                }
            }),
        )
        .expect("file update");
        assert!(matches!(
            &events[0],
            TelegramProviderEvent::FileChanged(file)
                if file.provider_file_id == "42" && file.downloaded_size_bytes == Some(40) && file.is_downloading
        ));
    }

    #[test]
    fn keeps_downloaded_file_path_only_on_the_private_runtime_update() {
        let updates = parse_provider_updates(
            "account",
            &json!({
                "@type": "updateFile",
                "file": {
                    "@type": "file",
                    "id": 42,
                    "size": 100,
                    "local": {
                        "path": "/private/provider/file",
                        "downloaded_size": 100,
                        "is_downloading_active": false,
                        "is_downloading_completed": true
                    }
                }
            }),
        )
        .expect("downloaded file update");
        assert!(matches!(
            &updates[0],
            TdlibProviderUpdate::Operational(event)
                if matches!(event.as_ref(), TelegramProviderEvent::FileChanged(file) if file.is_downloaded)
        ));
        assert!(matches!(
            &updates[1],
            TdlibProviderUpdate::DownloadedFile(file)
                if file.snapshot.provider_file_id == "42"
                    && file.local_path == PathBuf::from("/private/provider/file")
        ));
        assert!(!format!("{:?}", updates[1]).contains("/private/provider/file"));
    }
}

#[cfg(test)]
mod call_update_tests {
    use super::*;

    #[test]
    fn parses_pending_call_without_promoting_volatile_id_to_persistent_identity() {
        let updates = parse_provider_updates(
            "account-1",
            &json!({
                "@type": "updateCall",
                "call": {
                    "id": 41,
                    "unique_id": 0,
                    "user_id": 99,
                    "is_outgoing": false,
                    "is_video": false,
                    "state": {
                        "@type": "callStatePending",
                        "is_created": true,
                        "is_received": false
                    }
                }
            }),
        )
        .expect("pending call");

        assert!(matches!(
            updates.as_slice(),
            [TdlibProviderUpdate::Call(TdlibCallObservation {
                account_id,
                tdlib_call_id: 41,
                provider_call_unique_id: None,
                provider_user_id,
                direction: TdlibCallDirection::Incoming,
                is_video: false,
                state: TdlibCallState::Pending,
                pending_created: true,
                pending_received: false,
                discard_reason: None,
                failure_category: None,
            })] if account_id == "account-1" && provider_user_id == "99"
        ));
    }

    #[test]
    fn parses_ready_signaling_and_discarded_states_without_exposing_secrets() {
        let encryption_key = vec![7_u8; CALL_ENCRYPTION_KEY_BYTES];
        let peer_tag = [8_u8; 16];
        let ready = parse_provider_updates(
            "account-1",
            &json!({
                "@type": "updateCall",
                "call": {
                    "id": 41,
                    "unique_id": 5001,
                    "user_id": 99,
                    "is_outgoing": true,
                    "is_video": false,
                    "state": {
                        "@type": "callStateReady",
                        "protocol": {
                            "@type": "callProtocol",
                            "udp_p2p": true,
                            "udp_reflector": true,
                            "min_layer": 65,
                            "max_layer": 92,
                            "library_versions": ["pinned-tgcalls"]
                        },
                        "servers": [{
                            "@type": "callServer",
                            "id": 4,
                            "ip_address": "127.0.0.1",
                            "ipv6_address": "",
                            "port": 443,
                            "type": {
                                "@type": "callServerTypeTelegramReflector",
                                "peer_tag": STANDARD.encode(peer_tag),
                                "is_tcp": true
                            }
                        }],
                        "config": "private-config",
                        "encryption_key": STANDARD.encode(&encryption_key),
                        "custom_parameters": "private-parameters",
                        "allow_p2p": true
                    }
                }
            }),
        )
        .expect("ready call");
        let debug = format!("{ready:?}");
        assert!(!debug.contains("private-config"));
        assert!(!debug.contains(&STANDARD.encode(&encryption_key)));
        assert!(!debug.contains("private-parameters"));
        let [
            TdlibProviderUpdate::CallReady {
                observation,
                material,
            },
        ] = ready.as_slice()
        else {
            panic!("ready call update");
        };
        assert_eq!(observation.state, TdlibCallState::Ready);
        assert_eq!(material.servers.len(), 1);
        assert!(material.allow_tcp);

        let signaling = parse_provider_updates(
            "account-1",
            &json!({
                "@type": "updateNewCallSignalingData",
                "call_id": 41,
                "data": STANDARD.encode(b"private-signaling")
            }),
        )
        .expect("call signaling");
        let signaling_debug = format!("{signaling:?}");
        assert!(!signaling_debug.contains("private-signaling"));
        assert!(matches!(
            signaling.as_slice(),
            [TdlibProviderUpdate::CallSignaling {
                account_id,
                tdlib_call_id: 41,
                data,
            }] if account_id == "account-1" && data.expose() == b"private-signaling"
        ));

        let discarded = parse_provider_updates(
            "account-1",
            &json!({
                "@type": "updateCall",
                "call": {
                    "id": 41,
                    "unique_id": 5001,
                    "user_id": 99,
                    "is_outgoing": true,
                    "is_video": false,
                    "state": {
                        "@type": "callStateDiscarded",
                        "reason": {"@type": "callDiscardReasonMissed"}
                    }
                }
            }),
        )
        .expect("discarded call");

        assert!(matches!(
            discarded.as_slice(),
            [TdlibProviderUpdate::Call(TdlibCallObservation {
                state: TdlibCallState::Discarded,
                discard_reason: Some(TdlibCallDiscardReason::Missed),
                ..
            })]
        ));
    }

    #[test]
    fn unknown_call_state_fails_closed() {
        let error = parse_provider_updates(
            "account-1",
            &json!({
                "@type": "updateCall",
                "call": {
                    "id": 41,
                    "unique_id": 0,
                    "user_id": 99,
                    "is_outgoing": false,
                    "is_video": false,
                    "state": {"@type": "callStateFuture"}
                }
            }),
        )
        .expect_err("unknown call state");

        assert!(matches!(error, TdlibError::Protocol(_)));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TdlibError {
    Transport(String),
    Protocol(String),
    AuthenticationRequired,
    RuntimeUnavailable,
}

type TdJsonClientCreate = unsafe extern "C" fn() -> *mut c_void;
type TdJsonClientSend = unsafe extern "C" fn(*mut c_void, *const c_char);
type TdJsonClientReceive = unsafe extern "C" fn(*mut c_void, f64) -> *const c_char;
type TdJsonClientExecute = unsafe extern "C" fn(*mut c_void, *const c_char) -> *const c_char;
type TdJsonClientDestroy = unsafe extern "C" fn(*mut c_void);

/// Loaded libtdjson handle. The unsafe C ABI is isolated to this adapter file.
#[derive(Clone)]
pub struct TdJsonLibrary {
    inner: Arc<TdJsonLibraryInner>,
}

struct TdJsonLibraryInner {
    create: TdJsonClientCreate,
    send: TdJsonClientSend,
    receive: TdJsonClientReceive,
    execute: TdJsonClientExecute,
    destroy: TdJsonClientDestroy,
    _library: Library,
}

impl TdJsonLibrary {
    /// Loads only the exact signed TDLib artifact selected by the integration.
    /// No host-library discovery is permitted for a managed runtime.
    pub fn load_exact(path: &Path) -> Result<Self, TdlibError> {
        if !path.is_absolute() {
            return Err(TdlibError::Protocol(
                "TDLib artifact path is not absolute".to_owned(),
            ));
        }
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|_| TdlibError::Protocol("TDLib artifact is unavailable".to_owned()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(TdlibError::Protocol("TDLib artifact is invalid".to_owned()));
        }
        let library = unsafe { Library::new(path) }
            .map_err(|_| TdlibError::Protocol("TDLib artifact is unavailable".to_owned()))?;
        Self::from_library(library, path)
    }

    pub fn load(configured_path: Option<&Path>) -> Result<Self, TdlibError> {
        let candidates = library_candidates(configured_path);
        let mut errors = Vec::new();
        for candidate in candidates {
            let library = unsafe { Library::new(&candidate) };
            match library {
                Ok(library) => return Self::from_library(library, &candidate),
                Err(error) => errors.push(format!("{}: {error}", candidate.display())),
            }
        }
        Err(TdlibError::Transport(format!(
            "unable to load libtdjson: {}",
            errors.join("; ")
        )))
    }

    fn from_library(library: Library, candidate: &Path) -> Result<Self, TdlibError> {
        Ok(Self {
            inner: Arc::new(TdJsonLibraryInner {
                create: load_symbol(&library, b"td_json_client_create\0", candidate)?,
                send: load_symbol(&library, b"td_json_client_send\0", candidate)?,
                receive: load_symbol(&library, b"td_json_client_receive\0", candidate)?,
                execute: load_symbol(&library, b"td_json_client_execute\0", candidate)?,
                destroy: load_symbol(&library, b"td_json_client_destroy\0", candidate)?,
                _library: library,
            }),
        })
    }

    pub fn create_client(&self) -> Result<TdJsonClient, TdlibError> {
        let client = unsafe { (self.inner.create)() };
        if client.is_null() {
            return Err(TdlibError::Transport(
                "td_json_client_create returned null".to_owned(),
            ));
        }
        let client = TdJsonClient {
            client,
            library: self.clone(),
        };
        client.execute_json(&disable_tdlib_logging_request())?;
        Ok(client)
    }
}

pub struct TdJsonClient {
    client: *mut c_void,
    library: TdJsonLibrary,
}

impl TdJsonClient {
    pub fn send_json(&self, request: &Value) -> Result<(), TdlibError> {
        let request = CString::new(request.to_string())
            .map_err(|_| TdlibError::Protocol("TDLib request contains NUL".to_owned()))?;
        unsafe { (self.library.inner.send)(self.client, request.as_ptr()) };
        Ok(())
    }

    pub fn receive_json(&self, timeout_seconds: f64) -> Result<Option<Value>, TdlibError> {
        let response = unsafe { (self.library.inner.receive)(self.client, timeout_seconds) };
        parse_response(response)
    }

    pub fn execute_json(&self, request: &Value) -> Result<Option<Value>, TdlibError> {
        let request = CString::new(request.to_string())
            .map_err(|_| TdlibError::Protocol("TDLib request contains NUL".to_owned()))?;
        let response = unsafe { (self.library.inner.execute)(self.client, request.as_ptr()) };
        parse_response(response)
    }
}

impl Drop for TdJsonClient {
    fn drop(&mut self) {
        if !self.client.is_null() {
            unsafe { (self.library.inner.destroy)(self.client) };
            self.client = std::ptr::null_mut();
        }
    }
}

fn parse_response(response: *const c_char) -> Result<Option<Value>, TdlibError> {
    if response.is_null() {
        return Ok(None);
    }
    let text = unsafe { CStr::from_ptr(response) }
        .to_str()
        .map_err(|error| TdlibError::Protocol(format!("invalid TDLib UTF-8: {error}")))?;
    serde_json::from_str(text)
        .map(Some)
        .map_err(|error| TdlibError::Protocol(format!("invalid TDLib JSON: {error}")))
}

fn load_symbol<T: Copy>(
    library: &Library,
    name: &'static [u8],
    candidate: &Path,
) -> Result<T, TdlibError> {
    let symbol = unsafe { library.get::<T>(name) }.map_err(|error| {
        TdlibError::Transport(format!(
            "libtdjson `{}` is missing symbol: {error}",
            candidate.display()
        ))
    })?;
    Ok(*symbol)
}

fn library_candidates(configured_path: Option<&Path>) -> Vec<PathBuf> {
    if let Some(path) = configured_path {
        return vec![path.to_path_buf()];
    }
    #[cfg(target_os = "macos")]
    {
        vec![
            PathBuf::from("libtdjson.dylib"),
            PathBuf::from("/opt/homebrew/opt/tdlib/lib/libtdjson.dylib"),
            PathBuf::from("/usr/local/opt/tdlib/lib/libtdjson.dylib"),
        ]
    }
    #[cfg(target_os = "linux")]
    {
        return vec![
            PathBuf::from("libtdjson.so"),
            PathBuf::from("/usr/local/lib/libtdjson.so"),
            PathBuf::from("/usr/lib/libtdjson.so"),
        ];
    }
    #[cfg(target_os = "windows")]
    {
        vec![PathBuf::from("tdjson.dll")]
    }
}

#[derive(Debug, Eq, PartialEq)]
struct FolderReassignmentPlan {
    added_provider_folder_ids: Vec<i64>,
    removed_provider_folder_ids: Vec<i64>,
}

fn provider_folder_ids_from_chat(payload: &Value) -> Result<Vec<i64>, TdlibError> {
    if payload.get("@type").and_then(Value::as_str) != Some("chat") {
        return Err(TdlibError::Protocol(
            "TDLib getChat response is missing chat payload".to_owned(),
        ));
    }
    let mut folder_ids = Vec::new();
    for position in payload
        .get("positions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(list) = position.get("list") else {
            continue;
        };
        if list.get("@type").and_then(Value::as_str) != Some("chatListFolder") {
            continue;
        }
        let folder_id = list
            .get("chat_folder_id")
            .and_then(Value::as_i64)
            .ok_or_else(|| {
                TdlibError::Protocol("TDLib folder chat list is missing folder id".to_owned())
            })?;
        if !folder_ids.contains(&folder_id) {
            folder_ids.push(folder_id);
        }
    }
    normalized_folder_ids(&folder_ids)
}

fn normalized_folder_ids(folder_ids: &[i64]) -> Result<Vec<i64>, TdlibError> {
    if folder_ids.iter().any(|folder_id| *folder_id <= 0) {
        return Err(TdlibError::Protocol(
            "TDLib folder set contains an invalid folder id".to_owned(),
        ));
    }
    let mut normalized = folder_ids.to_vec();
    normalized.sort_unstable();
    normalized.dedup();
    Ok(normalized)
}

fn plan_folder_reassignment(
    current_provider_folder_ids: &[i64],
    target_provider_folder_ids: &[i64],
) -> FolderReassignmentPlan {
    FolderReassignmentPlan {
        added_provider_folder_ids: target_provider_folder_ids
            .iter()
            .copied()
            .filter(|folder_id| !current_provider_folder_ids.contains(folder_id))
            .collect(),
        removed_provider_folder_ids: current_provider_folder_ids
            .iter()
            .copied()
            .filter(|folder_id| !target_provider_folder_ids.contains(folder_id))
            .collect(),
    }
}

/// The runtime owns this port; TDLib transport implementations own sockets/processes.
pub trait TdlibTransport {
    fn request(&mut self, request: TdlibRequest) -> Result<TdlibResponse, TdlibError>;
    fn poll_updates(&mut self) -> Result<Vec<TdlibProviderUpdate>, TdlibError>;
}

/// Real libtdjson execution port. It owns correlation, while provider parsing stays here.
pub struct TdJsonTransport {
    client: TdJsonClient,
    account_id: String,
    receive_timeout_seconds: f64,
    request_timeout: Duration,
    pending_updates: VecDeque<Value>,
    known_chats: HashMap<String, TelegramChat>,
    known_user_names: HashMap<String, String>,
    correlation_sequence: u64,
    own_provider_user_id: Option<String>,
}

impl TdJsonTransport {
    pub fn new(client: TdJsonClient, account_id: impl Into<String>) -> Result<Self, TdlibError> {
        let account_id = account_id.into();
        if account_id.trim().is_empty() {
            return Err(TdlibError::Protocol(
                "Telegram account id is empty".to_owned(),
            ));
        }
        Ok(Self {
            client,
            account_id,
            receive_timeout_seconds: 0.25,
            request_timeout: Duration::from_secs(30),
            pending_updates: VecDeque::new(),
            known_chats: HashMap::new(),
            known_user_names: HashMap::new(),
            correlation_sequence: 0,
            own_provider_user_id: None,
        })
    }

    pub fn with_timeouts(
        mut self,
        receive_timeout_seconds: f64,
        request_timeout: Duration,
    ) -> Result<Self, TdlibError> {
        if !(0.0..=10.0).contains(&receive_timeout_seconds) || request_timeout.is_zero() {
            return Err(TdlibError::Protocol(
                "TDLib transport timeout is invalid".to_owned(),
            ));
        }
        self.receive_timeout_seconds = receive_timeout_seconds;
        self.request_timeout = request_timeout;
        Ok(self)
    }

    pub fn poll_updates(&mut self) -> Result<Vec<TdlibProviderUpdate>, TdlibError> {
        let mut updates = Vec::new();
        let pending_payloads = drain_pending_update_payloads(
            &mut self.pending_updates,
            MAX_TDLIB_UPDATE_PAYLOADS_PER_POLL,
        );
        let mut processed_payloads = pending_payloads.len();
        for mut payload in pending_payloads {
            self.remember_chat_update(&payload)?;
            self.remember_sender_update(&payload)?;
            self.enrich_message_sender_names(&mut payload)?;
            updates.extend(parse_provider_updates(&self.account_id, &payload)?);
        }
        while processed_payloads < MAX_TDLIB_UPDATE_PAYLOADS_PER_POLL {
            let Some(mut payload) = self.client.receive_json(0.0)? else {
                break;
            };
            self.remember_chat_update(&payload)?;
            self.remember_sender_update(&payload)?;
            self.enrich_message_sender_names(&mut payload)?;
            updates.extend(parse_provider_updates(&self.account_id, &payload)?);
            processed_payloads += 1;
        }
        Ok(updates)
    }

    fn remember_chat_update(&mut self, payload: &Value) -> Result<(), TdlibError> {
        if let Some(chat) = parse_new_chat_update(&self.account_id, payload)? {
            if let Some(provider_user_id) = private_chat_provider_user_id(payload) {
                self.known_user_names
                    .insert(provider_user_id, chat.title.clone());
            }
            self.known_chats.insert(chat.provider_chat_id.clone(), chat);
        }
        if payload.get("@type").and_then(Value::as_str) == Some("updateChatPhoto") {
            let avatar = parse_chat_avatar(&self.account_id, payload)?;
            if let Some(chat) = self.known_chats.get_mut(&avatar.provider_chat_id) {
                chat.avatar_provider_file_id = avatar.provider_file_id;
                chat.avatar_provider_unique_id = avatar.provider_unique_id;
            }
        }
        Ok(())
    }

    fn remember_sender_update(&mut self, payload: &Value) -> Result<(), TdlibError> {
        match payload.get("@type").and_then(Value::as_str) {
            Some("updateUser") => {
                if let Some(user) = payload.get("user") {
                    self.remember_user(user)?;
                }
            }
            Some("user") => self.remember_user(payload)?,
            Some("updateChatTitle") => {
                let provider_chat_id = required_string(payload, "chat_id")?;
                if let Some(chat) = self.known_chats.get_mut(&provider_chat_id)
                    && let Some(title) = bounded_sender_display_name(payload.get("title"))
                {
                    chat.title = title;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn remember_user(&mut self, user: &Value) -> Result<(), TdlibError> {
        let provider_user_id = required_string(user, "id")?;
        if let Some(display_name) = tdlib_user_display_name(user) {
            self.known_user_names.insert(provider_user_id, display_name);
        }
        Ok(())
    }

    fn enrich_message_sender_names(&mut self, payload: &mut Value) -> Result<(), TdlibError> {
        let sender_keys = message_sender_keys(payload);
        let mut lookup_failures = 0usize;
        let mut unresolved_users = 0usize;
        let mut unresolved_chats = 0usize;
        let mut first_failure = None;
        for sender in &sender_keys {
            if self.cached_sender_display_name(&sender).is_none() {
                // Sender labels improve presentation but never make message
                // history unavailable when TDLib can't resolve a deleted or
                // otherwise inaccessible peer.
                if let Err(error) = self.resolve_sender_display_name(&sender) {
                    lookup_failures = lookup_failures.saturating_add(1);
                    first_failure.get_or_insert(error);
                }
            }
            if self.cached_sender_display_name(sender).is_none() {
                match sender {
                    TdlibMessageSenderKey::User(_) => {
                        unresolved_users = unresolved_users.saturating_add(1)
                    }
                    TdlibMessageSenderKey::Chat(_) => {
                        unresolved_chats = unresolved_chats.saturating_add(1)
                    }
                }
            }
        }
        if unresolved_users > 0 || unresolved_chats > 0 {
            eprintln!(
                "developer_telegram_sender_resolution_unavailable total={} lookup_failures={} unresolved_users={} unresolved_chats={} first_failure={:?}",
                sender_keys.len(),
                lookup_failures,
                unresolved_users,
                unresolved_chats,
                first_failure,
            );
        }
        enrich_message_payloads(payload, &self.known_user_names, &self.known_chats);
        Ok(())
    }

    fn cached_sender_display_name(&self, sender: &TdlibMessageSenderKey) -> Option<&str> {
        match sender {
            TdlibMessageSenderKey::User(provider_user_id) => self
                .known_user_names
                .get(provider_user_id)
                .map(String::as_str),
            TdlibMessageSenderKey::Chat(provider_chat_id) => self
                .known_chats
                .get(provider_chat_id)
                .map(|chat| chat.title.as_str())
                .filter(|title| !title.trim().is_empty()),
        }
    }

    fn resolve_sender_display_name(
        &mut self,
        sender: &TdlibMessageSenderKey,
    ) -> Result<(), TdlibError> {
        let extra = self.next_correlation_extra("sender-lookup");
        let request = match sender {
            TdlibMessageSenderKey::User(provider_user_id) => json!({
                "@type": "getUser",
                "user_id": signed_chat_id(provider_user_id)?,
                "@extra": extra,
            }),
            TdlibMessageSenderKey::Chat(provider_chat_id) => json!({
                "@type": "getChat",
                "chat_id": signed_chat_id(provider_chat_id)?,
                "@extra": extra,
            }),
        };
        self.client.send_json(&request)?;
        let response = self.receive_correlated(&extra)?;
        self.remember_chat_update(&response)?;
        self.remember_sender_update(&response)
    }

    fn next_correlation_extra(&mut self, operation: &str) -> String {
        self.correlation_sequence = self.correlation_sequence.saturating_add(1);
        format!("telegram-{operation}-{}", self.correlation_sequence)
    }

    fn receive_correlated(&mut self, expected_extra: &str) -> Result<Value, TdlibError> {
        let started = Instant::now();
        while started.elapsed() < self.request_timeout {
            let Some(payload) = self.client.receive_json(self.receive_timeout_seconds)? else {
                continue;
            };
            if payload.get("@type").and_then(Value::as_str) == Some("error") {
                return Err(tdlib_error(&payload));
            }
            if payload.get("@extra").and_then(Value::as_str) == Some(expected_extra) {
                return Ok(payload);
            }
            self.remember_chat_update(&payload)?;
            self.remember_sender_update(&payload)?;
            self.pending_updates.push_back(payload);
        }
        Err(TdlibError::Protocol(format!(
            "TDLib request `{expected_extra}` timed out"
        )))
    }

    fn request_once(&mut self, request: &TdlibRequest) -> Result<TdlibResponse, TdlibError> {
        let payload = encode_request(request)?;
        let expected_extra = request_extra(request);
        self.client.send_json(&payload)?;
        let mut response = self.receive_correlated(&expected_extra)?;
        self.remember_chat_update(&response)?;
        self.remember_sender_update(&response)?;
        self.enrich_message_sender_names(&mut response)?;
        let parsed = parse_response_for_request(&self.account_id, request, response.clone())?;
        if is_download_file_request(request)
            && let Some(update) = completed_download_response_update(&self.account_id, &response)?
        {
            self.pending_updates.push_back(update);
        }
        Ok(parsed)
    }

    fn resolve_own_provider_user_id(&mut self) -> Result<String, TdlibError> {
        if let Some(provider_user_id) = &self.own_provider_user_id {
            return Ok(provider_user_id.clone());
        }
        let request = TdlibRequest::GetOwnUser {
            correlation_id: format!("telegram-own-user-{}", self.account_id),
        };
        let TdlibResponse::OwnUser { provider_user_id } = self.request_once(&request)? else {
            return Err(TdlibError::Protocol(
                "TDLib getMe returned an unexpected response".to_owned(),
            ));
        };
        self.own_provider_user_id = Some(provider_user_id.clone());
        Ok(provider_user_id)
    }

    fn request_participants(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        filter: TelegramParticipantFilter,
        offset: u32,
        limit: u32,
    ) -> Result<TdlibResponse, TdlibError> {
        let chat_extra = self.next_correlation_extra("participants-chat");
        self.client.send_json(&json!({
            "@type": "getChat",
            "chat_id": signed_chat_id(provider_chat_id)?,
            "@extra": chat_extra,
        }))?;
        let chat = self.receive_correlated(&chat_extra)?;
        self.remember_chat_update(&chat)?;
        let chat_type = chat.get("type").ok_or_else(|| {
            TdlibError::Protocol("TDLib participant chat type is missing".to_owned())
        })?;
        let chat_type_name = chat_type.get("@type").and_then(Value::as_str);
        let mut response = match chat_type_name {
            Some("chatTypeSupergroup") => {
                let supergroup_id = chat_type
                    .get("supergroup_id")
                    .and_then(value_id_optional)
                    .ok_or_else(|| {
                        TdlibError::Protocol(
                            "TDLib participant supergroup id is missing".to_owned(),
                        )
                    })?;
                let extra = self.next_correlation_extra("participants-supergroup");
                self.client.send_json(&json!({
                    "@type": "getSupergroupMembers",
                    "supergroup_id": provider_id(&supergroup_id)?,
                    "filter": {"@type": match filter {
                        TelegramParticipantFilter::Recent => "supergroupMembersFilterRecent",
                        TelegramParticipantFilter::Administrators => "supergroupMembersFilterAdministrators",
                    }},
                    "offset": offset,
                    "limit": limit,
                    "@extra": extra,
                }))?;
                self.receive_correlated(&extra)?
            }
            Some("chatTypeBasicGroup") => {
                let basic_group_id = chat_type
                    .get("basic_group_id")
                    .and_then(Value::as_i64)
                    .and_then(|value| i32::try_from(value).ok())
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                        TdlibError::Protocol(
                            "TDLib participant basic-group id is missing".to_owned(),
                        )
                    })?;
                let extra = self.next_correlation_extra("participants-basic-group");
                self.client.send_json(&json!({
                    "@type": "getBasicGroupFullInfo",
                    "basic_group_id": basic_group_id,
                    "@extra": extra,
                }))?;
                self.receive_correlated(&extra)?
            }
            Some("chatTypePrivate") => {
                let provider_user_id = chat_type
                    .get("user_id")
                    .and_then(value_id_optional)
                    .ok_or_else(|| {
                        TdlibError::Protocol(
                            "TDLib private-chat participant id is missing".to_owned(),
                        )
                    })?;
                let sender = TdlibMessageSenderKey::User(provider_user_id.clone());
                let _ = self.resolve_sender_display_name(&sender);
                let members = if offset == 0 && limit > 0 {
                    vec![json!({
                        "@type": "chatMember",
                        "member_id": {"@type": "messageSenderUser", "user_id": provider_id(&provider_user_id)?},
                        "status": {"@type": "chatMemberStatusMember"},
                    })]
                } else {
                    Vec::new()
                };
                json!({"@type": "chatMembers", "total_count": members.len(), "members": members})
            }
            Some("chatTypeSecret") => {
                let secret_chat_id = chat_type
                    .get("secret_chat_id")
                    .and_then(Value::as_i64)
                    .and_then(|value| i32::try_from(value).ok())
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                        TdlibError::Protocol(
                            "TDLib secret-chat participant id is missing".to_owned(),
                        )
                    })?;
                let extra = self.next_correlation_extra("participants-secret-chat");
                self.client.send_json(&json!({
                    "@type": "getSecretChat",
                    "secret_chat_id": secret_chat_id,
                    "@extra": extra,
                }))?;
                let secret_chat = self.receive_correlated(&extra)?;
                let provider_user_id = secret_chat
                    .get("user_id")
                    .and_then(value_id_optional)
                    .ok_or_else(|| {
                        TdlibError::Protocol("TDLib secret-chat user id is missing".to_owned())
                    })?;
                let sender = TdlibMessageSenderKey::User(provider_user_id.clone());
                let _ = self.resolve_sender_display_name(&sender);
                let members = if offset == 0 && limit > 0 {
                    vec![json!({
                        "@type": "chatMember",
                        "member_id": {"@type": "messageSenderUser", "user_id": provider_id(&provider_user_id)?},
                        "status": {"@type": "chatMemberStatusMember"},
                    })]
                } else {
                    Vec::new()
                };
                json!({"@type": "chatMembers", "total_count": members.len(), "members": members})
            }
            _ => {
                return Err(TdlibError::Protocol(
                    "TDLib participant chat kind is unsupported".to_owned(),
                ));
            }
        };
        self.remember_sender_update(&response)?;
        self.enrich_message_sender_names(&mut response)?;
        Ok(TdlibResponse::Participants(parse_participant_page(
            account_id,
            provider_chat_id,
            filter,
            offset,
            &response,
        )?))
    }

    fn reassign_chat_folders(
        &mut self,
        request: &TdlibRequest,
        operation_id: &str,
        provider_chat_id: &str,
        target_provider_folder_ids: &[i64],
    ) -> Result<TdlibResponse, TdlibError> {
        let chat_id = signed_chat_id(provider_chat_id)?;
        let target_provider_folder_ids = normalized_folder_ids(target_provider_folder_ids)?;
        let get_chat_extra = format!("{operation_id}:get-chat");
        self.client.send_json(&json!({
            "@type": "getChat",
            "chat_id": chat_id,
            "@extra": get_chat_extra,
        }))?;
        let chat = self.receive_correlated(&get_chat_extra)?;
        let current_provider_folder_ids = provider_folder_ids_from_chat(&chat)?;
        let plan =
            plan_folder_reassignment(&current_provider_folder_ids, &target_provider_folder_ids);

        for provider_folder_id in &plan.added_provider_folder_ids {
            let extra = format!("{operation_id}:add:{provider_folder_id}");
            self.client.send_json(&json!({
                "@type": "addChatToList",
                "chat_id": chat_id,
                "chat_list": {
                    "@type": "chatListFolder",
                    "chat_folder_id": provider_folder_id,
                },
                "@extra": extra,
            }))?;
            let response = self.receive_correlated(&extra)?;
            parse_response_for_request(&self.account_id, request, response)?;
        }
        for provider_folder_id in &plan.removed_provider_folder_ids {
            let get_folder_extra = format!("{operation_id}:remove:{provider_folder_id}:get");
            self.client.send_json(&json!({
                "@type": "getChatFolder",
                "chat_folder_id": provider_folder_id,
                "@extra": get_folder_extra,
            }))?;
            let folder = self.receive_correlated(&get_folder_extra)?;
            let edit_extra = format!("{operation_id}:remove:{provider_folder_id}");
            let edit =
                encode_remove_chat_from_folder(*provider_folder_id, chat_id, &folder, &edit_extra)?;
            self.client.send_json(&edit)?;
            let response = self.receive_correlated(&edit_extra)?;
            parse_response_for_request(&self.account_id, request, response)?;
        }

        let verify_chat_extra = format!("{operation_id}:verify-chat");
        self.client.send_json(&json!({
            "@type": "getChat",
            "chat_id": chat_id,
            "@extra": verify_chat_extra,
        }))?;
        let verified_chat = self.receive_correlated(&verify_chat_extra)?;
        if provider_folder_ids_from_chat(&verified_chat)? != target_provider_folder_ids {
            return Err(TdlibError::Protocol(
                "TDLib folder reassignment did not converge".to_owned(),
            ));
        }

        Ok(TdlibResponse::FolderReassigned {
            added_provider_folder_ids: plan.added_provider_folder_ids,
            removed_provider_folder_ids: plan.removed_provider_folder_ids,
        })
    }
}

fn drain_pending_update_payloads(
    pending_updates: &mut VecDeque<Value>,
    limit: usize,
) -> Vec<Value> {
    let drain_count = pending_updates.len().min(limit);
    pending_updates.drain(..drain_count).collect()
}

impl TdlibTransport for TdJsonTransport {
    fn request(&mut self, request: TdlibRequest) -> Result<TdlibResponse, TdlibError> {
        if let TdlibRequest::ProviderCommand(TelegramProviderCommand::ReassignChatFolders {
            operation_id,
            provider_chat_id,
            target_provider_folder_ids,
            ..
        }) = &request
        {
            return self.reassign_chat_folders(
                &request,
                operation_id,
                provider_chat_id,
                target_provider_folder_ids,
            );
        }
        if let TdlibRequest::ProviderCommand(TelegramProviderCommand::RemoveChatFromFolder {
            operation_id,
            provider_chat_id,
            provider_folder_id,
            ..
        }) = &request
        {
            let get_extra = format!("{operation_id}:get");
            self.client.send_json(&json!({
                "@type": "getChatFolder",
                "chat_folder_id": provider_folder_id,
                "@extra": get_extra,
            }))?;
            let folder = self.receive_correlated(&get_extra)?;
            let edit = encode_remove_chat_from_folder(
                *provider_folder_id,
                signed_chat_id(provider_chat_id)?,
                &folder,
                operation_id,
            )?;
            self.client.send_json(&edit)?;
            let response = self.receive_correlated(operation_id)?;
            return parse_response_for_request(&self.account_id, &request, response);
        }
        if let TdlibRequest::ListParticipants {
            account_id,
            provider_chat_id,
            filter,
            offset,
            limit,
        } = &request
        {
            return self.request_participants(
                account_id,
                provider_chat_id,
                *filter,
                *offset,
                *limit,
            );
        }
        if let TdlibRequest::ListBasicGroupParticipants {
            account_id,
            basic_group_id,
            ..
        } = &request
        {
            let group_extra = format!("telegram-basic-group-{account_id}-{basic_group_id}");
            self.client.send_json(&json!({
                "@type": "getBasicGroup",
                "basic_group_id": basic_group_id,
                "@extra": group_extra,
            }))?;
            let _group = self.receive_correlated(&group_extra)?;
            let full_info_extra =
                format!("telegram-basic-group-full-info-{account_id}-{basic_group_id}");
            self.client.send_json(&json!({
                "@type": "getBasicGroupFullInfo",
                "basic_group_id": basic_group_id,
                "@extra": full_info_extra,
            }))?;
            let response = self.receive_correlated(&full_info_extra)?;
            return parse_response_for_request(&self.account_id, &request, response);
        }
        if let TdlibRequest::LoadChats { account_id, limit } = &request {
            let max_pages = limit.div_ceil(100);
            let mut known_chat_count = self.known_chats.len();
            for page in 0..max_pages {
                let load_extra = format!("telegram-load-chats-{account_id}-{page}");
                self.client
                    .send_json(&encode_load_chats_page_request(100, &load_extra))?;
                let load_response = self.receive_correlated(&load_extra)?;
                validate_load_chats_page_response(&load_response)?;
                let exhausted = load_chats_page_exhausted(&load_response);
                let next_known_chat_count = self.known_chats.len();
                if !should_load_next_chat_page(
                    known_chat_count,
                    next_known_chat_count,
                    *limit as usize,
                    exhausted,
                ) {
                    break;
                }
                known_chat_count = next_known_chat_count;
            }

            let list_request = TdlibRequest::LoadChats {
                account_id: account_id.clone(),
                limit: (*limit).min(100),
            };
            let payload = encode_request(&list_request)?;
            let expected_extra = request_extra(&list_request);
            self.client.send_json(&payload)?;
            let response = self.receive_correlated(&expected_extra)?;
            let ids = response
                .get("chat_ids")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    TdlibError::Protocol("TDLib getChats response is missing chat_ids".to_owned())
                })?;
            let mut chats =
                Vec::with_capacity((*limit as usize).min(self.known_chats.len() + ids.len()));
            for id in ids.iter().take(100) {
                let provider_chat_id = value_id(id)?;
                if let Some(chat) = self.known_chats.get(&provider_chat_id).cloned() {
                    chats.push(chat);
                    continue;
                }
                let extra = format!("telegram-get-chat-{account_id}-{provider_chat_id}");
                self.client.send_json(&json!({
                    "@type": "getChat",
                    "chat_id": signed_chat_id(&provider_chat_id)?,
                    "@extra": extra,
                }))?;
                let chat_payload = self.receive_correlated(&extra)?;
                let chat = parse_chat(account_id, &chat_payload)?;
                self.known_chats
                    .insert(chat.provider_chat_id.clone(), chat.clone());
                chats.push(chat);
            }
            let mut remaining = self
                .known_chats
                .values()
                .filter(|chat| {
                    !chats
                        .iter()
                        .any(|existing| existing.provider_chat_id == chat.provider_chat_id)
                })
                .cloned()
                .collect::<Vec<_>>();
            remaining.sort_by(|left, right| left.provider_chat_id.cmp(&right.provider_chat_id));
            chats.extend(remaining);
            chats.truncate(*limit as usize);
            let own_provider_user_id = self.resolve_own_provider_user_id()?;
            label_saved_messages_chat(&mut chats, &own_provider_user_id);
            if let Some(chat) = self.known_chats.get_mut(&own_provider_user_id)
                && chat.kind == TelegramChatKind::Private
            {
                chat.title = "Saved Messages".to_owned();
            }
            if chats.is_empty() && *limit > 0 {
                return Err(TdlibError::Protocol(
                    "TDLib did not expose any loaded chats".to_owned(),
                ));
            }
            return Ok(TdlibResponse::Chats(chats));
        }
        self.request_once(&request)
    }

    fn poll_updates(&mut self) -> Result<Vec<TdlibProviderUpdate>, TdlibError> {
        TdJsonTransport::poll_updates(self)
    }
}

fn label_saved_messages_chat(chats: &mut [TelegramChat], own_provider_user_id: &str) {
    if let Some(chat) = chats.iter_mut().find(|chat| {
        chat.kind == TelegramChatKind::Private && chat.provider_chat_id == own_provider_user_id
    }) {
        chat.title = "Saved Messages".to_owned();
    }
}

fn should_load_next_chat_page(
    previous_known_chat_count: usize,
    current_known_chat_count: usize,
    requested_limit: usize,
    exhausted: bool,
) -> bool {
    !exhausted
        && current_known_chat_count > previous_known_chat_count
        && current_known_chat_count < requested_limit
}

fn encode_load_chats_page_request(limit: u32, extra: &str) -> Value {
    json!({
        "@type": "loadChats",
        "chat_list": null,
        "limit": limit,
        "@extra": extra,
    })
}

fn validate_load_chats_page_response(response: &Value) -> Result<(), TdlibError> {
    match response.get("@type").and_then(Value::as_str) {
        Some("ok") => Ok(()),
        Some("error") if response.get("code").and_then(Value::as_i64) == Some(404) => Ok(()),
        Some("error") => Err(tdlib_error(response)),
        _ => Err(TdlibError::Protocol(
            "TDLib loadChats response is invalid".to_owned(),
        )),
    }
}

fn load_chats_page_exhausted(response: &Value) -> bool {
    response.get("@type").and_then(Value::as_str) == Some("error")
        && response.get("code").and_then(Value::as_i64) == Some(404)
}

fn encode_remove_chat_from_folder(
    folder_id: i64,
    chat_id: i64,
    folder: &Value,
    extra: &str,
) -> Result<Value, TdlibError> {
    if folder.get("@type").and_then(Value::as_str) != Some("chatFolder") {
        return Err(TdlibError::Protocol(
            "TDLib getChatFolder response is missing chatFolder payload".to_owned(),
        ));
    }
    let unique_ids = |key: &str| {
        let mut ids = Vec::new();
        if let Some(values) = folder.get(key).and_then(Value::as_array) {
            for value in values.iter().filter_map(Value::as_i64) {
                if !ids.contains(&value) {
                    ids.push(value);
                }
            }
        }
        ids
    };
    let mut pinned = unique_ids("pinned_chat_ids");
    pinned.retain(|value| *value != chat_id);
    let mut included = unique_ids("included_chat_ids");
    included.retain(|value| *value != chat_id);
    let mut excluded = unique_ids("excluded_chat_ids");
    if !excluded.contains(&chat_id) {
        excluded.push(chat_id);
    }
    let text = |parent: &str, key: &str| {
        folder
            .get(parent)
            .and_then(|value| value.get(key))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("")
    };
    let bool_field = |key: &str| folder.get(key).and_then(Value::as_bool).unwrap_or(false);
    Ok(json!({
        "@type": "editChatFolder",
        "chat_folder_id": folder_id,
        "folder": {
            "@type": "chatFolder",
            "name": {"@type": "chatFolderName", "text": text("name", "text"), "animate_custom_emoji": folder.get("name").and_then(|value| value.get("animate_custom_emoji")).and_then(Value::as_bool).unwrap_or(false)},
            "icon": {"@type": "chatFolderIcon", "name": text("icon", "name")},
            "color_id": folder.get("color_id").and_then(Value::as_i64).unwrap_or_default(),
            "is_shareable": bool_field("is_shareable"),
            "pinned_chat_ids": pinned,
            "included_chat_ids": included,
            "excluded_chat_ids": excluded,
            "exclude_muted": bool_field("exclude_muted"),
            "exclude_read": bool_field("exclude_read"),
            "exclude_archived": bool_field("exclude_archived"),
            "include_contacts": bool_field("include_contacts"),
            "include_non_contacts": bool_field("include_non_contacts"),
            "include_bots": bool_field("include_bots"),
            "include_groups": bool_field("include_groups"),
            "include_channels": bool_field("include_channels")
        },
        "@extra": extra.trim()
    }))
}

fn request_extra(request: &TdlibRequest) -> String {
    match request {
        TdlibRequest::GetOwnUser { correlation_id }
        | TdlibRequest::ResolveSender { correlation_id, .. } => correlation_id.clone(),
        TdlibRequest::CreateCall { operation_id, .. }
        | TdlibRequest::AcceptCall { operation_id, .. }
        | TdlibRequest::DiscardCall { operation_id, .. } => operation_id.clone(),
        TdlibRequest::SendCallSignalingData { correlation_id, .. } => correlation_id.clone(),
        TdlibRequest::LoadChats { account_id, .. }
        | TdlibRequest::LoadHistory { account_id, .. }
        | TdlibRequest::ListTopics { account_id, .. }
        | TdlibRequest::ListParticipants { account_id, .. }
        | TdlibRequest::ListBasicGroupParticipants { account_id, .. } => account_id.clone(),
        TdlibRequest::GetMessage {
            account_id,
            provider_message_id,
            ..
        } => format!("{account_id}:message:{provider_message_id}"),
        TdlibRequest::GetChatFolder {
            account_id,
            provider_folder_id,
        } => format!("{account_id}:folder:{provider_folder_id}"),
        TdlibRequest::SendMessage(command) => command.operation_id.clone(),
        TdlibRequest::SendMedia(command) => command.operation_id.clone(),
        TdlibRequest::SendMediaMaterialized { command, .. } => command.operation_id.clone(),
        TdlibRequest::DownloadFile(command) => command.operation_id.clone(),
        TdlibRequest::ProviderCommand(command) => provider_command_operation_id(command).to_owned(),
    }
}

fn parse_response_for_request(
    account_id: &str,
    request: &TdlibRequest,
    response: Value,
) -> Result<TdlibResponse, TdlibError> {
    if response.get("@type").and_then(Value::as_str) == Some("error") {
        return Err(tdlib_error(&response));
    }
    match request {
        TdlibRequest::GetOwnUser { .. } => Ok(TdlibResponse::OwnUser {
            provider_user_id: value_id(
                response
                    .get("id")
                    .ok_or_else(|| TdlibError::Protocol("TDLib user is missing id".to_owned()))?,
            )?,
        }),
        TdlibRequest::ResolveSender {
            provider_sender_id, ..
        } => {
            let display_name = match response.get("@type").and_then(Value::as_str) {
                Some("user") => tdlib_user_display_name(&response),
                Some("chat") => bounded_sender_display_name(response.get("title")),
                _ => {
                    return Err(TdlibError::Protocol(
                        "TDLib sender lookup returned an unexpected response".to_owned(),
                    ));
                }
            };
            Ok(TdlibResponse::SenderName {
                provider_sender_id: provider_sender_id.clone(),
                display_name,
            })
        }
        TdlibRequest::CreateCall { operation_id, .. } => {
            let tdlib_call_id = response
                .get("id")
                .and_then(Value::as_i64)
                .and_then(|value| i32::try_from(value).ok())
                .filter(|value| *value > 0)
                .ok_or_else(|| {
                    TdlibError::Protocol("TDLib createCall response is missing id".to_owned())
                })?;
            Ok(TdlibResponse::CallCreated {
                operation_id: operation_id.clone(),
                tdlib_call_id,
            })
        }
        TdlibRequest::AcceptCall { operation_id, .. }
        | TdlibRequest::DiscardCall { operation_id, .. } => Ok(TdlibResponse::Accepted {
            operation_id: operation_id.clone(),
        }),
        TdlibRequest::SendCallSignalingData { correlation_id, .. } => Ok(TdlibResponse::Accepted {
            operation_id: correlation_id.clone(),
        }),
        TdlibRequest::LoadHistory { .. } => {
            let messages = response
                .get("messages")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    TdlibError::Protocol("TDLib history response is missing messages".to_owned())
                })?;
            Ok(TdlibResponse::History(
                messages
                    .iter()
                    .map(|message| parse_message_observation(account_id, message))
                    .collect::<Result<Vec<_>, _>>()?,
            ))
        }
        TdlibRequest::GetMessage { .. } => Ok(TdlibResponse::Message(parse_message_observation(
            account_id, &response,
        )?)),
        TdlibRequest::ListParticipants {
            provider_chat_id,
            filter,
            ..
        } => Ok(TdlibResponse::Participants(parse_participant_page(
            account_id,
            provider_chat_id,
            *filter,
            0,
            &response,
        )?)),
        TdlibRequest::ListBasicGroupParticipants {
            provider_chat_id, ..
        } => Ok(TdlibResponse::Participants(parse_participant_page(
            account_id,
            provider_chat_id,
            TelegramParticipantFilter::Recent,
            0,
            &response,
        )?)),
        TdlibRequest::ListTopics {
            provider_chat_id, ..
        } => Ok(TdlibResponse::Topics(parse_topic_list(
            account_id,
            provider_chat_id,
            &response,
        )?)),
        TdlibRequest::GetChatFolder { .. } => Ok(TdlibResponse::ChatFolders(parse_chat_folders(
            account_id,
            &json!({"chat_folders": [response]}),
        )?)),
        TdlibRequest::DownloadFile { .. } => Ok(TdlibResponse::File(parse_file_snapshot(
            account_id, &response,
        )?)),
        TdlibRequest::SendMessage(command) => sent_response(&command.operation_id, &response),
        TdlibRequest::SendMedia(command) => sent_response(&command.operation_id, &response),
        TdlibRequest::SendMediaMaterialized { command, .. } => {
            sent_response(&command.operation_id, &response)
        }
        TdlibRequest::ProviderCommand(TelegramProviderCommand::SearchMessages { .. }) => Ok(
            TdlibResponse::History(parse_message_list_response(account_id, &response)?),
        ),
        TdlibRequest::ProviderCommand(command) => {
            if response.get("@type").and_then(Value::as_str) == Some("message") {
                sent_response(provider_command_operation_id(command), &response)
            } else {
                Ok(TdlibResponse::Accepted {
                    operation_id: provider_command_operation_id(command).to_owned(),
                })
            }
        }
        TdlibRequest::LoadChats { .. } => {
            unreachable!("LoadChats is handled by TdJsonTransport::request")
        }
    }
}

fn parse_message_list_response(
    account_id: &str,
    response: &Value,
) -> Result<Vec<TelegramMessageObservation>, TdlibError> {
    let messages = response
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TdlibError::Protocol("TDLib search response is missing messages".to_owned())
        })?;
    messages
        .iter()
        .map(|message| parse_message_observation(account_id, message))
        .collect()
}

fn sent_response(operation_id: &str, response: &Value) -> Result<TdlibResponse, TdlibError> {
    if response.get("@type").and_then(Value::as_str) != Some("message") {
        return Ok(TdlibResponse::Accepted {
            operation_id: operation_id.to_owned(),
        });
    }
    Ok(TdlibResponse::Sent {
        provider_message_id: required_string(response, "id")?,
    })
}

fn tdlib_error(payload: &Value) -> TdlibError {
    let code = payload
        .get("code")
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_owned());
    TdlibError::Protocol(format!("TDLib error {code}"))
}

#[cfg(test)]
mod message_reference_tests {
    use super::*;

    #[test]
    fn parses_reply_and_forward_references_as_provider_data() {
        let observation = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 200,
                "sender_id": {"user_id": 42},
                "date": 10,
                "is_outgoing": false,
                "reply_to": {
                    "@type": "messageReplyToMessage",
                    "chat_id": 100,
                    "message_id": 150
                },
                "forward_info": {
                    "date": 5,
                    "origin": {
                        "@type": "messageOriginUser",
                        "sender_user_id": 7
                    }
                },
                "content": {"@type": "messageText", "text": {"text": "forwarded reply"}}
            }),
        )
        .expect("message references");

        assert_eq!(
            observation.references.reply_to,
            Some(TelegramReplyReference {
                provider_chat_id: "100".to_owned(),
                provider_message_id: "150".to_owned(),
            })
        );
        assert_eq!(
            observation.references.forward_origin,
            Some(TelegramForwardOrigin {
                provider_chat_id: None,
                provider_message_id: None,
                provider_sender_id: Some("7".to_owned()),
                sender_name: None,
                observed_at_unix_seconds: Some(5),
            })
        );
    }

    #[test]
    fn parses_legacy_channel_forward_origin() {
        let observation = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 200,
                "sender_id": {"user_id": 42},
                "date": 10,
                "is_outgoing": false,
                "forward_info": {
                    "date": 5,
                    "origin": {
                        "@type": "messageForwardOriginChannel",
                        "chat_id": -100700,
                        "message_id": 33
                    }
                },
                "content": {"@type": "messageText", "text": {"text": "forwarded"}}
            }),
        )
        .expect("legacy channel forward origin");

        assert_eq!(
            observation
                .references
                .forward_origin
                .and_then(|origin| origin.provider_sender_id),
            Some("-100700".to_owned()),
        );
    }
}

pub struct TdlibClient<T> {
    transport: T,
}

#[cfg(test)]
mod folder_command_tests {
    use super::*;

    #[test]
    fn add_chat_to_folder_uses_provider_folder_chat_list() {
        let command = TelegramProviderCommand::AddChatToFolder {
            operation_id: "op-folder-add".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            provider_folder_id: 7,
        };
        let encoded = encode_provider_command(&command).expect("valid folder add");
        assert_eq!(encoded["@type"], "addChatToList");
        assert_eq!(encoded["chat_id"], 100);
        assert_eq!(encoded["chat_list"]["@type"], "chatListFolder");
        assert_eq!(encoded["chat_list"]["chat_folder_id"], 7);
    }

    #[test]
    fn remove_chat_from_folder_preserves_folder_policy_and_updates_membership() {
        let folder = json!({
            "@type": "chatFolder",
            "name": {"text": " Projects ", "animate_custom_emoji": true},
            "icon": {"name": "briefcase"},
            "color_id": 3,
            "is_shareable": true,
            "pinned_chat_ids": [100, 101],
            "included_chat_ids": [100, 102],
            "excluded_chat_ids": [103],
            "exclude_muted": true,
            "exclude_read": false,
            "exclude_archived": true,
            "include_contacts": true,
            "include_non_contacts": false,
            "include_bots": true,
            "include_groups": true,
            "include_channels": false
        });
        let encoded = encode_remove_chat_from_folder(7, 100, &folder, "op-folder-remove")
            .expect("valid folder removal");
        assert_eq!(encoded["@type"], "editChatFolder");
        assert_eq!(encoded["folder"]["name"]["text"], "Projects");
        assert_eq!(encoded["folder"]["name"]["animate_custom_emoji"], true);
        assert_eq!(encoded["folder"]["pinned_chat_ids"], json!([101]));
        assert_eq!(encoded["folder"]["included_chat_ids"], json!([102]));
        assert_eq!(encoded["folder"]["excluded_chat_ids"], json!([103, 100]));
        assert_eq!(encoded["folder"]["exclude_muted"], true);
        assert_eq!(encoded["folder"]["include_bots"], true);
    }

    #[test]
    fn folder_reassignment_converges_from_current_provider_membership() {
        let chat = json!({
            "@type": "chat",
            "id": 100,
            "positions": [
                {"list": {"@type": "chatListMain"}, "order": 1},
                {"list": {"@type": "chatListFolder", "chat_folder_id": 9}, "order": 3},
                {"list": {"@type": "chatListFolder", "chat_folder_id": 7}, "order": 2},
                {"list": {"@type": "chatListFolder", "chat_folder_id": 9}, "order": 4}
            ]
        });
        let current = provider_folder_ids_from_chat(&chat).expect("provider folder memberships");

        assert_eq!(current, vec![7, 9]);
        let target = normalized_folder_ids(&[11, 9]).expect("normalized target");
        assert_eq!(
            plan_folder_reassignment(&current, &target),
            FolderReassignmentPlan {
                added_provider_folder_ids: vec![11],
                removed_provider_folder_ids: vec![7],
            }
        );
    }

    #[test]
    fn folder_reassignment_command_starts_with_a_correlated_chat_snapshot() {
        let command = TelegramProviderCommand::ReassignChatFolders {
            operation_id: "op-folder-reassign".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            target_provider_folder_ids: vec![9, 11],
        };
        let encoded = encode_provider_command(&command).expect("valid folder reassignment");

        assert_eq!(encoded["@type"], "getChat");
        assert_eq!(encoded["chat_id"], 100);
        assert_eq!(encoded["@extra"], "op-folder-reassign:get-chat");
    }

    #[test]
    fn parses_search_message_list_into_provider_observations() {
        let messages = parse_message_list_response(
            "account",
            &json!({
                "@type": "messages",
                "messages": [{
                    "chat_id": 100,
                    "id": 200,
                    "sender_id": {"user_id": 42},
                    "date": 10,
                    "is_outgoing": false,
                    "content": {"@type": "messageText", "text": {"text": "release"}}
                }]
            }),
        )
        .expect("search messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].provider_chat_id, "100");
        assert_eq!(messages[0].provider_message_id, "200");
        assert_eq!(messages[0].text.as_deref(), Some("release"));
    }
}

#[cfg(test)]
mod call_command_tests {
    use makosh_telegram_call_media_contract::TelegramCallProtocolV1;

    use super::*;

    fn protocol() -> TelegramCallProtocolV1 {
        TelegramCallProtocolV1::new(true, true, vec!["pinned-tgcalls".to_owned()])
            .expect("protocol")
    }

    #[test]
    fn encodes_exact_audio_call_signaling_requests() {
        let create = encode_request(&TdlibRequest::CreateCall {
            operation_id: "call-create".to_owned(),
            provider_user_id: "9001".to_owned(),
            protocol: protocol(),
        })
        .expect("createCall");
        assert_eq!(create["@type"], "createCall");
        assert_eq!(create["user_id"], 9001);
        assert_eq!(create["is_video"], false);
        assert_eq!(create["protocol"]["@type"], "callProtocol");
        assert_eq!(create["protocol"]["min_layer"], 65);
        assert_eq!(create["protocol"]["max_layer"], 92);
        assert_eq!(create["protocol"]["library_versions"][0], "pinned-tgcalls");

        let accept = encode_request(&TdlibRequest::AcceptCall {
            operation_id: "call-accept".to_owned(),
            tdlib_call_id: 41,
            protocol: protocol(),
        })
        .expect("acceptCall");
        assert_eq!(accept["@type"], "acceptCall");
        assert_eq!(accept["call_id"], 41);
        assert_eq!(accept["protocol"]["udp_p2p"], true);

        let discard = encode_request(&TdlibRequest::DiscardCall {
            operation_id: "call-discard".to_owned(),
            tdlib_call_id: 41,
            is_disconnected: false,
            duration_seconds: 37,
            connection_id: 9002,
        })
        .expect("discardCall");
        assert_eq!(discard["@type"], "discardCall");
        assert_eq!(discard["duration"], 37);
        assert_eq!(discard["connection_id"], 9002);
        assert_eq!(discard["invite_link"], "");
        assert_eq!(discard["is_video"], false);

        let signaling = encode_request(&TdlibRequest::SendCallSignalingData {
            correlation_id: "call-signal-1".to_owned(),
            tdlib_call_id: 41,
            data: TelegramCallSecretBytesV1::new(
                b"private-signaling".to_vec(),
                MAX_SIGNALING_DATA_BYTES,
            )
            .expect("signaling"),
        })
        .expect("sendCallSignalingData");
        assert_eq!(signaling["@type"], "sendCallSignalingData");
        assert_eq!(signaling["call_id"], 41);
        assert_eq!(signaling["data"], STANDARD.encode(b"private-signaling"));
        assert_eq!(signaling["@extra"], "call-signal-1");
    }

    #[test]
    fn parses_create_call_id_and_sanitizes_correlated_provider_errors() {
        let request = TdlibRequest::CreateCall {
            operation_id: "call-create".to_owned(),
            provider_user_id: "9001".to_owned(),
            protocol: protocol(),
        };
        assert_eq!(
            parse_response_for_request(
                "account",
                &request,
                json!({"@type": "callId", "id": 41, "@extra": "call-create"}),
            ),
            Ok(TdlibResponse::CallCreated {
                operation_id: "call-create".to_owned(),
                tdlib_call_id: 41,
            })
        );
        let error = parse_response_for_request(
            "account",
            &request,
            json!({
                "@type": "error",
                "code": 400,
                "message": "private provider detail",
                "@extra": "call-create"
            }),
        )
        .expect_err("provider error");
        assert_eq!(error, TdlibError::Protocol("TDLib error 400".to_owned()));
    }
}

impl<T> TdlibClient<T>
where
    T: TdlibTransport,
{
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn request(&mut self, request: TdlibRequest) -> Result<TdlibResponse, TdlibError> {
        self.transport.request(request)
    }
}
