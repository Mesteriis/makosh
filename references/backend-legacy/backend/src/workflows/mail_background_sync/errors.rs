use makosh_communications_api::accounts::ProviderAccountPortError;
use makosh_communications_api::evidence::CommunicationEvidencePortError;
use makosh_events_api::EventEnvelopeError;
use thiserror::Error;

use crate::platform::communications::email_sync::EmailSyncPlanError;
use crate::workflows::email_sync_pipeline::errors::EmailSyncPipelineError;
use crate::workflows::graph_projection::errors::GraphProjectionError;
use makosh_communications_api::mail_resources::EmailProviderSyncError;
use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum MailSyncError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    CommunicationEvidence(#[from] CommunicationEvidencePortError),

    #[error(transparent)]
    EmailSyncPlan(#[from] EmailSyncPlanError),

    #[error(transparent)]
    ProviderSync(#[from] EmailProviderSyncError),

    #[error(transparent)]
    ProviderAccount(#[from] ProviderAccountPortError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    ObservationPort(#[from] ObservationStoreError),

    #[error("mail sync account was not found")]
    AccountNotFound,

    #[error("mail sync run is already active for account")]
    RunAlreadyActive,

    #[error("mail sync run was not found")]
    RunNotFound,

    #[error("invalid mail sync setting {field}: {message}")]
    InvalidSetting {
        field: &'static str,
        message: &'static str,
    },
}

#[derive(Debug, Error)]
pub(super) enum ProviderSyncError {
    #[error(transparent)]
    CommunicationEvidence(#[from] CommunicationEvidencePortError),

    #[error(transparent)]
    ProviderSync(#[from] EmailProviderSyncError),

    #[error(transparent)]
    Pipeline(#[from] EmailSyncPipelineError),

    #[error(transparent)]
    Graph(#[from] GraphProjectionError),

    #[error(transparent)]
    SyncState(#[from] MailSyncError),
}
