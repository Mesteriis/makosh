use chrono::{DateTime, Utc};
use makosh_communications_api::commands::NewCommunicationProviderCommand;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;
use uuid::Uuid;

use super::ai_state::{
    CommunicationAiStateRecord, CommunicationAiStateStore, CommunicationAiStateTransitionRequest,
};
use super::drafts::{
    CommunicationDraft, CommunicationDraftError, CommunicationDraftStore, DraftStatus,
    NewCommunicationDraft,
};
use super::flags::{MessageFlags, MessageFlagsError};
use super::folders::{
    CommunicationFolder, CommunicationFolderError, CommunicationFolderStore,
    FolderMessageActionResponse, NewCommunicationFolder, UpdateCommunicationFolder,
};
use super::messages::errors::MessageProjectionError;
use super::messages::models::ProjectedMessage;
use super::messages::store::MessageProjectionStore;
use super::outbox::provider_send_store::ProviderSendStoreError;
use super::outbox::{
    CommunicationOutboxError, CommunicationOutboxItem, CommunicationOutboxStatus,
    CommunicationOutboxStore, NewCommunicationOutboxItem,
};
use super::saved_searches::{
    CommunicationSavedSearch, CommunicationSavedSearchError, CommunicationSavedSearchStore,
    NewCommunicationSavedSearch, UpdateCommunicationSavedSearch,
};
use super::storage::errors::{AttachmentSafetyScanError, CommunicationStorageError};
use crate::domains::communications::evidence::merge_metadata;
use makosh_communications_api::accounts::ProviderAccount;
use makosh_communications_api::email::OutgoingEmail;
use makosh_communications_postgres::provider_commands::{
    CommunicationProviderCommandError, CommunicationProviderCommandStore,
};
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

const LOCAL_USER_ACTOR_ID: &str = "makosh-local-user";

#[derive(Clone)]
pub struct CommunicationCommandService {
    pub(super) pool: PgPool,
}

impl CommunicationCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Clone, Debug)]
pub struct CommunicationDraftUpsertCommand {
    pub draft_id: String,
    pub account_id: String,
    pub persona_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Option<Vec<String>>,
    pub bcc_recipients: Option<Vec<String>>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Option<Vec<String>>,
    pub attachment_ids: Option<Vec<String>>,
    pub status: Option<String>,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct CommunicationAttachmentImportCommand {
    pub account_id: Option<String>,
    pub channel_kind: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub content_base64: String,
    pub source_kind: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct CommunicationOutboxSendCommand {
    pub draft_id: Option<String>,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub undo_send_seconds: Option<i64>,
    pub metadata: Value,
}

#[derive(Clone, Debug)]
pub struct CommunicationWorkflowStateTransitionResult {
    pub updated: ProjectedMessage,
    pub previous_state: String,
}

#[derive(Debug, Error)]
pub enum CommunicationCommandServiceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{operation} observation capture failed")]
    ObservationCapture {
        operation: &'static str,
        #[source]
        source: ObservationStoreError,
    },

    #[error("{0}")]
    InvalidRequest(&'static str),

    #[error(transparent)]
    Draft(#[from] CommunicationDraftError),

    #[error(transparent)]
    Folder(#[from] CommunicationFolderError),

    #[error(transparent)]
    SavedSearch(#[from] CommunicationSavedSearchError),

    #[error(transparent)]
    Outbox(#[from] CommunicationOutboxError),

    #[error(transparent)]
    CommunicationStorage(#[from] CommunicationStorageError),

    #[error(transparent)]
    AttachmentScan(#[from] AttachmentSafetyScanError),

    #[error(transparent)]
    ProviderSendStore(#[from] ProviderSendStoreError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error(transparent)]
    CommunicationAiState(#[from] super::ai_state::CommunicationAiStateError),

    #[error(transparent)]
    MessageFlags(#[from] MessageFlagsError),

    #[error(transparent)]
    ProviderCommand(#[from] CommunicationProviderCommandError),
}
mod drafts;
mod folders;
mod message_state;
mod outbox;
mod saved_searches;
