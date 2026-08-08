use crate::platform::audit::store::ApiAuditLog;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, Router as ConnectRouter, ServiceRequest,
    ServiceResult,
};
use sqlx::postgres::PgPool;

use super::communications_request_policy::invalid_argument_error;
use super::communications_request_policy::normalize_limit;
use super::communications_request_policy::{
    parse_draft_status, parse_local_state, parse_match_mode, parse_outbox_status,
    parse_workflow_state,
};
use crate::app::handlers::communications::workflow_actions::handler::execute_workflow_action;
use crate::application::communication_send::{
    CommunicationSendDependencies, CommunicationSendError, CommunicationSendRequest, send_email,
};
use crate::domains::communications::analytics::{EmailAnalyticsError, EmailAnalyticsStore};
use crate::domains::communications::archive_inspection::{
    ArchiveInspectionLimits, cached_archive_inspection_report, inspect_zip_bytes,
};
use crate::domains::communications::attachment_search::{
    AttachmentSearchError, AttachmentSearchQuery, AttachmentSearchStore,
};
use crate::domains::communications::bulk_actions::{
    BulkMessageActionError, BulkMessageActionStore,
};
use crate::domains::communications::command_service::{
    CommunicationCommandService, CommunicationDraftUpsertCommand,
};
use crate::domains::communications::drafts::{CommunicationDraftError, CommunicationDraftStore};
use crate::domains::communications::folders::{
    CommunicationFolderError, CommunicationFolderListQuery, CommunicationFolderStore,
    FolderMessageListQuery, NewCommunicationFolder, UpdateCommunicationFolder,
};
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::models::ProjectedMessagePageQuery;
use crate::domains::communications::messages::states::{LocalMessageState, WorkflowState};
use crate::domains::communications::messages::store::MessageProjectionStore;
use crate::domains::communications::outbox::{CommunicationOutboxError, CommunicationOutboxStore};
use crate::domains::communications::personas::{
    CommunicationPersonaError, CommunicationPersonaStore,
};
use crate::domains::communications::saved_searches::{
    CommunicationSavedSearchError, CommunicationSavedSearchListQuery,
    CommunicationSavedSearchStore, NewCommunicationSavedSearch, UpdateCommunicationSavedSearch,
};
use crate::domains::communications::storage::errors::CommunicationStorageError;
use crate::domains::communications::storage::models::StoredCommunicationAttachmentWithBlob;
use crate::domains::communications::storage::store::CommunicationStorageStore;
use crate::domains::communications::subscriptions::{SubscriptionError, SubscriptionStore};
use crate::domains::communications::templates::{
    CommunicationMergePreviewRow, CommunicationTemplateError, CommunicationTemplateStore,
    NewCommunicationTemplate,
};
use crate::domains::communications::threads::{CommunicationThreadError, CommunicationThreadStore};
use crate::platform::audit::models::NewApiAuditRecord;
use crate::platform::config::app_config::AppConfig;
use crate::vault::HostVault;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_connectrpc_contracts::makosh::common::v1::PageResponse;
use makosh_connectrpc_contracts::makosh::communications::v1::{
    AddMessageLabelResponse, AiReplyRequest as ProtoAiReplyRequest,
    AiReplyResponse as ProtoAiReplyResponse, AiReplyVariantsRequest as ProtoAiReplyVariantsRequest,
    AiReplyVariantsResponse as ProtoAiReplyVariantsResponse, AnalyzeMessageRequest,
    AnalyzeMessageResponse, AttachmentSearchRequest, AttachmentSearchResponse,
    BulkMessageActionRequest as ProtoBulkMessageActionRequest,
    BulkMessageActionResponse as ProtoBulkMessageActionResponse, CommunicationsService,
    CommunicationsServiceExt, CopyMessageToFolderRequest, CopyMessageToFolderResponse,
    CreateDraftRequest, CreateDraftResponse, CreateFolderRequest, CreateFolderResponse,
    CreateSavedSearchRequest, CreateSavedSearchResponse, DeleteDraftRequest, DeleteDraftResponse,
    DeleteFolderRequest, DeleteFolderResponse, DeleteMessageFromProviderRequest,
    DeleteMessageFromProviderResponse, DeleteRichTemplateRequest, DeleteRichTemplateResponse,
    DeleteSavedSearchRequest, DeleteSavedSearchResponse, DetectMessageLanguageRequest,
    DetectMessageLanguageResponse, ExplainMessageRequest, ExplainMessageResponse,
    ExtractAttachmentTextRequest, ExtractAttachmentTextResponse, ExtractMessageNotesRequest,
    ExtractMessageNotesResponse, ExtractMessageTasksRequest, ExtractMessageTasksResponse,
    GetAttachmentArchiveInspectionRequest, GetAttachmentArchiveInspectionResponse,
    GetAttachmentExtractedTextRequest, GetAttachmentExtractedTextResponse,
    GetAttachmentPreviewRequest, GetAttachmentPreviewResponse, GetMailboxHealthRequest,
    GetMailboxHealthResponse, GetMessageAuthRequest, GetMessageAuthResponse,
    GetMessageExportRequest, GetMessageExportResponse, GetMessageRequest, GetMessageResponse,
    GetMessageSignatureRequest, GetMessageSignatureResponse, GetMessageSmartCcRequest,
    GetMessageSmartCcResponse, ListCommunicationBlockersRequest, ListCommunicationBlockersResponse,
    ListCommunicationPersonasRequest, ListCommunicationPersonasResponse, ListDraftsRequest,
    ListDraftsResponse, ListFolderMessagesRequest, ListFolderMessagesResponse, ListFoldersRequest,
    ListFoldersResponse, ListMessageWorkflowStateCountsRequest,
    ListMessageWorkflowStateCountsResponse, ListMessagesRequest, ListMessagesResponse,
    ListOutboxRequest, ListOutboxResponse, ListRichTemplatesRequest, ListRichTemplatesResponse,
    ListSavedSearchesRequest, ListSavedSearchesResponse, ListSubscriptionsRequest,
    ListSubscriptionsResponse, ListThreadMessagesRequest, ListThreadMessagesResponse,
    ListThreadsRequest, ListThreadsResponse, ListTopSendersRequest, ListTopSendersResponse,
    MarkMessageReadRequest, MarkMessageReadResponse, MessageToggleRequest,
    MoveMessageToFolderRequest, MoveMessageToFolderResponse, RedirectMessageRequest,
    RedirectMessageResponse, RemoveMessageLabelResponse, RichTemplateMailMergePreviewRequest,
    RichTemplateMailMergePreviewResponse, RichTemplateRenderRequest, RichTemplateRenderResponse,
    SearchMessagesRequest, SearchMessagesResponse, SendMessageRequest, SendMessageResponse,
    SnoozeMessageRequest, SnoozeMessageResponse,
    ThreadTranslationItem as ProtoThreadTranslationItem, ToggleMessageImportantResponse,
    ToggleMessageMuteResponse, ToggleMessagePinResponse, TransitionMessageWorkflowStateRequest,
    TransitionMessageWorkflowStateResponse, TranslateAttachmentRequest,
    TranslateAttachmentResponse, TranslateMessageRequest, TranslateMessageResponse,
    TranslateThreadRequest, TranslateThreadResponse, UndoOutboxItemRequest, UndoOutboxItemResponse,
    UpdateFolderRequest, UpdateFolderResponse, UpdateMessageLabelRequest,
    UpdateMessageLocalStateRequest, UpdateMessageLocalStateResponse, UpdateSavedSearchRequest,
    UpdateSavedSearchResponse, UpsertRichTemplateRequest, UpsertRichTemplateResponse,
    WorkflowActionRequest as ProtoWorkflowActionRequest,
    WorkflowActionResponse as ProtoWorkflowActionResponse,
};

#[path = "communications_service_impl.rs"]
#[allow(refining_impl_trait)]
mod communications_service_impl;
use communications_service_impl::AttachmentPreviewKind;

const ATTACHMENT_TRANSLATION_SOURCE: &str = "durable_extracted_text";

pub(crate) fn register(
    router: ConnectRouter,
    pool: Option<PgPool>,
    config: AppConfig,
    vault: HostVault,
) -> ConnectRouter {
    let Some(pool) = pool else {
        return router;
    };

    Arc::new(CommunicationsConnectService::new(pool, config, vault)).register(router)
}

struct CommunicationsConnectService {
    config: AppConfig,
    pool: PgPool,
    vault: HostVault,
    message_store: MessageProjectionStore,
    provider_command_store: CommunicationProviderCommandStore,
    provider_account_store: CommunicationProviderAccountStore,
    analytics_store: EmailAnalyticsStore,
    subscription_store: SubscriptionStore,
    persona_store: CommunicationPersonaStore,
    template_store: CommunicationTemplateStore,
    thread_store: CommunicationThreadStore,
    draft_store: CommunicationDraftStore,
    outbox_store: CommunicationOutboxStore,
    storage_store: CommunicationStorageStore,
    attachment_search_store: AttachmentSearchStore,
    saved_search_store: CommunicationSavedSearchStore,
    folder_store: CommunicationFolderStore,
    audit_log: ApiAuditLog,
}

impl CommunicationsConnectService {
    fn new(pool: PgPool, config: AppConfig, vault: HostVault) -> Self {
        Self {
            config,
            pool: pool.clone(),
            vault,
            message_store: MessageProjectionStore::new(pool.clone()),
            provider_command_store: CommunicationProviderCommandStore::new(pool.clone()),
            provider_account_store: CommunicationProviderAccountStore::new(pool.clone()),
            analytics_store: EmailAnalyticsStore::new(pool.clone()),
            subscription_store: SubscriptionStore::new(pool.clone()),
            persona_store: CommunicationPersonaStore::new(pool.clone()),
            template_store: CommunicationTemplateStore::new(pool.clone()),
            thread_store: CommunicationThreadStore::new(pool.clone()),
            draft_store: CommunicationDraftStore::new(pool.clone()),
            outbox_store: CommunicationOutboxStore::new(pool.clone()),
            storage_store: CommunicationStorageStore::new(pool.clone()),
            attachment_search_store: AttachmentSearchStore::new(pool.clone()),
            saved_search_store: CommunicationSavedSearchStore::new(pool.clone()),
            folder_store: CommunicationFolderStore::new(pool.clone()),
            audit_log: ApiAuditLog::new(pool),
        }
    }
}

#[allow(refining_impl_trait)]
fn attachment_preview_kind(
    attachment: &StoredCommunicationAttachmentWithBlob,
) -> Option<AttachmentPreviewKind> {
    if super::communications_attachment_policy::is_previewable_text(attachment) {
        return Some(AttachmentPreviewKind::Text);
    }
    if super::communications_attachment_policy::image_content_type(attachment).is_some() {
        return Some(AttachmentPreviewKind::Image);
    }
    if super::communications_attachment_policy::audio_content_type(attachment).is_some() {
        return Some(AttachmentPreviewKind::Audio);
    }
    if super::communications_attachment_policy::video_content_type(attachment).is_some() {
        return Some(AttachmentPreviewKind::Video);
    }
    None
}

fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

pub(super) fn message_connect_error(error: MessageProjectionError) -> ConnectError {
    match error {
        MessageProjectionError::MessageNotFound => {
            ConnectError::new(ErrorCode::NotFound, error.to_string())
        }
        MessageProjectionError::InvalidLimit(_)
        | MessageProjectionError::InvalidCursor
        | MessageProjectionError::InvalidWorkflowState(_)
        | MessageProjectionError::InvalidLocalState(_)
        | MessageProjectionError::EmptyField(_)
        | MessageProjectionError::MissingPayloadField(_)
        | MessageProjectionError::InvalidMessageMetadata
        | MessageProjectionError::InvalidStoredRecipients
        | MessageProjectionError::UnsupportedRawBlobStorageKind(_)
        | MessageProjectionError::RawRecordTupleMismatch { .. }
        | MessageProjectionError::RawRecordNotFound(_)
        | MessageProjectionError::InvalidImportanceScore(_) => {
            invalid_argument_error(error.to_string())
        }
        MessageProjectionError::Sqlx(_)
        | MessageProjectionError::ProjectionWrite(_)
        | MessageProjectionError::CommunicationStorage(_)
        | MessageProjectionError::Rfc822(_)
        | MessageProjectionError::ObservationStore(_)
        | MessageProjectionError::ProviderCommand(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn thread_connect_error(error: CommunicationThreadError) -> ConnectError {
    match error {
        CommunicationThreadError::InvalidCursor => invalid_argument_error(error.to_string()),
        CommunicationThreadError::Sqlx(_) | CommunicationThreadError::Serde(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn draft_connect_error(error: CommunicationDraftError) -> ConnectError {
    match error {
        CommunicationDraftError::Invalid(_) | CommunicationDraftError::InvalidCursor => {
            invalid_argument_error(error.to_string())
        }
        CommunicationDraftError::Sqlx(_)
        | CommunicationDraftError::Observation(_)
        | CommunicationDraftError::EventStore(_)
        | CommunicationDraftError::EventEnvelope(_)
        | CommunicationDraftError::Serde(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn saved_search_connect_error(error: CommunicationSavedSearchError) -> ConnectError {
    match error {
        CommunicationSavedSearchError::Invalid(_)
        | CommunicationSavedSearchError::InvalidCursor => invalid_argument_error(error.to_string()),
        CommunicationSavedSearchError::Sqlx(_)
        | CommunicationSavedSearchError::Observation(_)
        | CommunicationSavedSearchError::Serde(_)
        | CommunicationSavedSearchError::EventStore(_)
        | CommunicationSavedSearchError::EventEnvelope(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn attachment_search_connect_error(error: AttachmentSearchError) -> ConnectError {
    match error {
        AttachmentSearchError::InvalidCursor => invalid_argument_error(error.to_string()),
        AttachmentSearchError::Sqlx(_) | AttachmentSearchError::CommunicationStorage(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn attachment_text_extraction_connect_error(
    error: crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionServiceError,
) -> ConnectError {
    match error {
        crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionServiceError::NotFound => {
            ConnectError::new(ErrorCode::NotFound, "attachment was not found")
        }
        crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionServiceError::Quarantined => {
            ConnectError::new(ErrorCode::FailedPrecondition, error.to_string())
        }
        crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionServiceError::UnsupportedStorage
        | crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionServiceError::Extraction(_)
        | crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionServiceError::InvalidDerivedText => {
            invalid_argument_error("attachment text extraction is unavailable")
        }
        crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionServiceError::Storage(error) => {
            storage_connect_error(error)
        }
        crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionServiceError::Sqlx(error) => {
            tracing::error!(error = %error, "attachment text extraction persistence failed");
            ConnectError::new(ErrorCode::Internal, "attachment text extraction failed")
        }
        crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionServiceError::Event(error) => {
            tracing::error!(error = %error, "attachment text extraction event persistence failed");
            ConnectError::new(ErrorCode::Internal, "attachment text extraction failed")
        }
        crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionServiceError::EventEnvelope(error) => {
            tracing::error!(error = %error, "attachment text extraction event construction failed");
            ConnectError::new(ErrorCode::Internal, "attachment text extraction failed")
        }
    }
}

fn attachment_safe_preview_connect_error(
    error: crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewServiceError,
) -> ConnectError {
    match error {
        crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewServiceError::NotFound => {
            ConnectError::new(ErrorCode::NotFound, "attachment was not found")
        }
        crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewServiceError::Quarantined => {
            ConnectError::new(
                ErrorCode::FailedPrecondition,
                "attachment preview is blocked by attachment scan status",
            )
        }
        crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewServiceError::UnsupportedStorage
        | crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewServiceError::Rendering(_)
        | crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewServiceError::InvalidArtifact => {
            invalid_argument_error("attachment safe preview is unavailable")
        }
        crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewServiceError::Storage(error) => {
            storage_connect_error(error)
        }
        crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewServiceError::Sqlx(error) => {
            tracing::error!(error = %error, "attachment safe preview persistence failed");
            ConnectError::new(ErrorCode::Internal, "attachment safe preview is unavailable")
        }
        crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewServiceError::Event(error) => {
            tracing::error!(error = %error, "attachment safe preview event persistence failed");
            ConnectError::new(ErrorCode::Internal, "attachment safe preview is unavailable")
        }
        crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewServiceError::EventEnvelope(error) => {
            tracing::error!(error = %error, "attachment safe preview event construction failed");
            ConnectError::new(ErrorCode::Internal, "attachment safe preview is unavailable")
        }
    }
}

fn bulk_action_connect_error(error: BulkMessageActionError) -> ConnectError {
    match error {
        BulkMessageActionError::Invalid(message) => invalid_argument_error(message),
        BulkMessageActionError::Sqlx(_)
        | BulkMessageActionError::ObservationStore(_)
        | BulkMessageActionError::EventStore(_)
        | BulkMessageActionError::EventEnvelope(_)
        | BulkMessageActionError::ProviderCommand(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn folder_connect_error(error: CommunicationFolderError) -> ConnectError {
    match error {
        CommunicationFolderError::Invalid(_) | CommunicationFolderError::InvalidCursor => {
            invalid_argument_error(error.to_string())
        }
        CommunicationFolderError::Sqlx(_)
        | CommunicationFolderError::Observation(_)
        | CommunicationFolderError::Serde(_)
        | CommunicationFolderError::EventStore(_)
        | CommunicationFolderError::EventEnvelope(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn outbox_connect_error(error: CommunicationOutboxError) -> ConnectError {
    match error {
        CommunicationOutboxError::Invalid(_)
        | CommunicationOutboxError::InvalidCursor
        | CommunicationOutboxError::UndoUnavailable => invalid_argument_error(error.to_string()),
        CommunicationOutboxError::Sqlx(_)
        | CommunicationOutboxError::Serde(_)
        | CommunicationOutboxError::EventStore(_)
        | CommunicationOutboxError::EventEnvelope(_)
        | CommunicationOutboxError::ObservationStore(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

pub(super) fn storage_connect_error(error: CommunicationStorageError) -> ConnectError {
    ConnectError::new(ErrorCode::Internal, error.to_string())
}

fn audit_connect_error(error: crate::platform::audit::errors::ApiAuditError) -> ConnectError {
    ConnectError::new(ErrorCode::Internal, error.to_string())
}

fn search_connect_error(
    error: crate::domains::communications::search::IndexEmailError,
) -> ConnectError {
    match error {
        crate::domains::communications::search::IndexEmailError::Messages(message_error) => {
            message_connect_error(message_error)
        }
        crate::domains::communications::search::IndexEmailError::Search(search_error) => {
            ConnectError::new(ErrorCode::Internal, search_error.to_string())
        }
    }
}

fn search_engine_connect_error(error: crate::engines::search::errors::SearchError) -> ConnectError {
    ConnectError::new(ErrorCode::Internal, error.to_string())
}

fn extract_connect_error(
    error: crate::domains::communications::extract::ExtractError,
) -> ConnectError {
    ConnectError::new(ErrorCode::Internal, error.to_string())
}

fn analytics_connect_error(error: EmailAnalyticsError) -> ConnectError {
    match error {
        EmailAnalyticsError::InvalidCursor => invalid_argument_error(error.to_string()),
        EmailAnalyticsError::Sqlx(_) | EmailAnalyticsError::Serde(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn subscription_connect_error(error: SubscriptionError) -> ConnectError {
    match error {
        SubscriptionError::InvalidCursor => invalid_argument_error(error.to_string()),
        SubscriptionError::Sqlx(_) | SubscriptionError::Serde(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn persona_connect_error(error: CommunicationPersonaError) -> ConnectError {
    match error {
        CommunicationPersonaError::Invalid(_) => invalid_argument_error(error.to_string()),
        CommunicationPersonaError::Sqlx(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn template_connect_error(error: CommunicationTemplateError) -> ConnectError {
    match error {
        CommunicationTemplateError::InvalidTemplate(_) => invalid_argument_error(error.to_string()),
        CommunicationTemplateError::Sqlx(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn ai_state_connect_error(
    error: crate::domains::communications::ai_state::CommunicationAiStateError,
) -> ConnectError {
    match error {
        crate::domains::communications::ai_state::CommunicationAiStateError::Invalid(_) => {
            invalid_argument_error(error.to_string())
        }
        other => ConnectError::new(ErrorCode::Internal, other.to_string()),
    }
}

pub(super) fn api_error_connect_error(error: crate::app::error::types::ApiError) -> ConnectError {
    let _ = error;
    ConnectError::new(ErrorCode::Internal, "internal API error")
}

fn send_connect_error(error: CommunicationSendError) -> ConnectError {
    match error {
        CommunicationSendError::InvalidRequest(_)
        | CommunicationSendError::ProviderAccountNotFound => {
            invalid_argument_error(error.to_string())
        }
        CommunicationSendError::CommunicationIngestion(_)
        | CommunicationSendError::Command(_)
        | CommunicationSendError::Audit(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn command_connect_error(
    error: crate::domains::communications::command_service::CommunicationCommandServiceError,
) -> ConnectError {
    match error {
        crate::domains::communications::command_service::CommunicationCommandServiceError::Draft(
            draft_error,
        ) => draft_connect_error(draft_error),
        crate::domains::communications::command_service::CommunicationCommandServiceError::Outbox(
            outbox_error,
        ) => outbox_connect_error(outbox_error),
        crate::domains::communications::command_service::CommunicationCommandServiceError::SavedSearch(
            saved_search_error,
        ) => saved_search_connect_error(saved_search_error),
        crate::domains::communications::command_service::CommunicationCommandServiceError::Folder(
            folder_error,
        ) => folder_connect_error(folder_error),
        crate::domains::communications::command_service::CommunicationCommandServiceError::MessageProjection(
            message_error,
        ) => message_connect_error(message_error),
        other => ConnectError::new(ErrorCode::Internal, other.to_string()),
    }
}
