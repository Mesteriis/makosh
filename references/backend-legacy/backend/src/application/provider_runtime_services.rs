use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::integrations::telegram::client::commands;
use crate::integrations::telegram::client::commands::queries::list_commands_filtered as list_telegram_commands_filtered;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::messages::attachments::TelegramAttachmentDownloadStateUpdate;
use crate::integrations::telegram::client::models::accounts::{
    TelegramAccount, TelegramAccountSetupRequest, TelegramAccountSetupResponse,
    TelegramLiveAccountSetupRequest,
};
use crate::integrations::telegram::client::models::chats::{TelegramChat, TelegramChatMember};
use crate::integrations::telegram::client::models::messages::{
    TelegramCommandListResponse, TelegramMessage, TelegramProviderWriteCommand,
};
use crate::integrations::telegram::client::models::topics::{
    TelegramTopic, TelegramTopicListResponse,
};
use crate::integrations::telegram::client::store::TelegramStore as TelegramProviderRuntimeStore;
use crate::integrations::telegram::client::topics::{
    get_topic as get_telegram_topic, list_topic_message_ids as list_telegram_topic_message_ids,
    list_topics as list_telegram_topics, search_topics as search_telegram_topics_projection,
};
use crate::integrations::telegram::client::vault::TelegramSecretVault;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::{
    WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebMessage, WhatsappWebSession,
};
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppAuthorizedSessionCredentialWrite, WhatsAppConversationCommandRequest,
    WhatsAppCredentialBinding, WhatsAppDeleteRequest, WhatsAppEditRequest, WhatsAppForwardRequest,
    WhatsAppMediaDownloadRequest, WhatsAppMediaUploadRequest, WhatsAppProviderCommand,
    WhatsAppProviderCommandListResponse, WhatsAppProviderCommandResponse, WhatsAppReactionRequest,
    WhatsAppReplyRequest, WhatsAppRuntimeHealth, WhatsAppRuntimeRelinkRequest,
    WhatsAppRuntimeRemoveRequest, WhatsAppRuntimeRemoveResponse, WhatsAppRuntimeRevokeRequest,
    WhatsAppRuntimeStartRequest, WhatsAppRuntimeStatus, WhatsAppRuntimeStopRequest,
    WhatsAppStatusPublishRequest, WhatsAppTextSendRequest, WhatsAppVoiceNoteSendRequest,
};
use crate::integrations::yandex_telemost::client::errors::YandexTelemostError;
use crate::integrations::yandex_telemost::client::models::{
    YandexTelemostAccountListResponse, YandexTelemostRetentionCleanupRequest,
    YandexTelemostRetentionCleanupResponse,
};
use crate::integrations::yandex_telemost::client::store::YandexTelemostStore;
use crate::integrations::zoom::client::errors::ZoomError;
use crate::integrations::zoom::client::models::oauth_models::{
    ZoomOAuthPendingGrant, ZoomOAuthStartRequest, ZoomServerToServerAuthorizeRequest,
    ZoomTokenMaintenanceRequest, ZoomTokenMaintenanceResult, ZoomTokenRefreshRequest,
    ZoomTokenRefreshResult,
};
use crate::integrations::zoom::client::models::{
    ZoomAccountListResponse, ZoomAccountSetupRequest, ZoomAccountSetupResponse,
    ZoomAuditEventResponse, ZoomAuthorizationResult, ZoomLiveAccountSetupRequest,
    ZoomMeetingIngestResult, ZoomMeetingObservationRequest, ZoomRecordingImportAuditResponse,
    ZoomRecordingImportRemoveRequest, ZoomRecordingImportRemoveResponse, ZoomRecordingIngestResult,
    ZoomRecordingMediaDownloadRequest, ZoomRecordingMediaImportResult,
    ZoomRecordingObservationRequest, ZoomRecordingSyncRequest, ZoomRecordingSyncResult,
    ZoomRetentionCleanupRequest, ZoomRetentionCleanupResponse, ZoomRuntimeRemoveRequest,
    ZoomRuntimeRemoveResponse, ZoomRuntimeStartRequest, ZoomRuntimeStatus, ZoomRuntimeStopRequest,
    ZoomTranscriptFileImportRequest, ZoomTranscriptFileImportResult, ZoomTranscriptIngestResult,
    ZoomTranscriptObservationRequest, ZoomWebhookSubscriptionReconcileRequest,
    ZoomWebhookSubscriptionReconcileResult, ZoomWebhookSubscriptionRemoveRequest,
    ZoomWebhookSubscriptionRemoveResult, ZoomWebhookSubscriptionStatusRequest,
    ZoomWebhookSubscriptionStatusResult,
};
use crate::integrations::zoom::client::store::ZoomStore as ZoomProviderRuntimeStore;
use crate::platform::secrets::store::SecretReferenceStore;
use crate::vault::HostVault;
use makosh_communications_api::accounts::ProviderAccount;

pub(crate) type WhatsAppProviderRuntimeRef =
    Arc<dyn crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderRuntime>;

#[derive(Clone)]
pub(crate) struct TelegramProviderRuntimeApplicationService {
    store: TelegramProviderRuntimeStore,
}

#[derive(Clone)]
pub(crate) struct ZoomProviderRuntimeApplicationService {
    store: ZoomProviderRuntimeStore,
}

#[derive(Clone)]
pub(crate) struct YandexTelemostProviderRuntimeApplicationService {
    store: YandexTelemostStore,
}

#[derive(Clone)]
pub(crate) struct WhatsappProviderRuntimeApplicationService {
    runtime: WhatsAppProviderRuntimeRef,
}

mod telegram;
mod telemost;
mod whatsapp;
mod zoom;
